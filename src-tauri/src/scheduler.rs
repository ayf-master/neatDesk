// 定时任务调度模块 — 根据配置自动整理桌面

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{self, Duration};
use tauri::AppHandle;
use crate::commands::AppState;

/// 调度器状态
pub struct Scheduler {
    running: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动定时调度（非阻塞）
    pub fn start(&self, app: AppHandle, state: Arc<std::sync::Mutex<AppState>>) {
        if self.running.load(Ordering::SeqCst) {
            return; // 已在运行
        }
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        // 在后台 tokio 任务中运行
        tauri::async_runtime::spawn(async move {
            run_scheduler(app, state, running).await;
        });
    }

    /// 停止调度
    /// 当前无调用方（应用退出时进程整体结束），保留作为公共 API 供未来动态启停
    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

async fn run_scheduler(_app: AppHandle, state: Arc<std::sync::Mutex<AppState>>, running: Arc<AtomicBool>) {
    // 每 30 秒检查一次配置，看是否需要执行
    let mut interval = time::interval(Duration::from_secs(30));
    // 已触发整理的日期；当天触发过一次后不再重复触发
    let mut last_trigger_date: Option<String> = None;

    while running.load(Ordering::SeqCst) {
        interval.tick().await;

        if !running.load(Ordering::SeqCst) {
            break;
        }

        // 读取配置
        let config = {
            if let Ok(state_guard) = state.lock() {
                Some(crate::config::load_config(&state_guard.config_dir))
            } else {
                None
            }
        };

        let config = match config {
            Some(c) => c,
            None => continue,
        };

        // 检查定时是否启用
        if !config.schedule.enabled {
            continue;
        }

        // 判断是否需要执行整理
        let now = chrono::Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        // 今天已触发过 → 当天不再触发（修复 ±5 分钟窗口内重复触发约 10 次的问题）
        let already_triggered_today = last_trigger_date.as_deref() == Some(today.as_str());

        let should_run = match config.schedule.schedule_type.as_str() {
            "interval" => {
                // 间隔模式按"距上次整理是否达到 N 小时"判断，不套用 already_triggered_today，
                // 否则会被限制在一天只触发一次，与 UI 上"每 N 小时整理一次"的语义矛盾。
                let should_organize = {
                    let history: Vec<serde_json::Value> = match state.lock() {
                        Ok(s) => crate::undo::get_history(&s.data_dir).unwrap_or_default(),
                        Err(_) => vec![],
                    };

                    match history.first() {
                        None => true, // 无历史记录，直接整理
                        Some(last) => {
                            // 检查是否已撤销
                            if last.get("undone").and_then(|v| v.as_bool()).unwrap_or(false) {
                                true
                            } else {
                                // 检查距离上次整理的时间间隔
                                match last.get("organized_at").and_then(|v| v.as_str()) {
                                    None => true,
                                    Some(time_str) => {
                                        match chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
                                            Ok(last_time) => {
                                                let elapsed = (now.naive_local() - last_time).num_hours();
                                                elapsed >= config.schedule.interval_hours as i64
                                            }
                                            Err(_) => true,
                                        }
                                    }
                                }
                            }
                        }
                    }
                };
                should_organize
            }
            "daily" => {
                // 每天指定时间执行
                if already_triggered_today {
                    false
                } else {
                    let target_time = format!("{} {}", today, config.schedule.daily_time);
                    if let Ok(target) = chrono::NaiveDateTime::parse_from_str(
                        &target_time,
                        "%Y-%m-%d %H:%M"
                    ) {
                        // 在当前时间的 ±5 分钟内
                        let diff = (now.naive_local() - target).num_minutes().abs();
                        diff <= 5
                    } else {
                        false
                    }
                }
            }
            "weekly" => {
                // 每周指定天 + 时间执行
                if already_triggered_today {
                    false
                } else {
                    let weekday = now.format("%A").to_string();
                    if weekday.eq_ignore_ascii_case(&config.schedule.weekly_day) {
                        let target_time = format!("{} {}", today, config.schedule.weekly_time);
                        if let Ok(target) = chrono::NaiveDateTime::parse_from_str(
                            &target_time,
                            "%Y-%m-%d %H:%M"
                        ) {
                            let diff = (now.naive_local() - target).num_minutes().abs();
                            diff <= 5
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            }
            _ => false,
        };

        // 执行整理
        if should_run {
            // 重启兜底：last_trigger_date 丢失后，查当天是否已有整理记录（含手动），有则跳过
            let has_record_today = {
                let history: Vec<serde_json::Value> = match state.lock() {
                    Ok(s) => crate::undo::get_history(&s.data_dir).unwrap_or_default(),
                    Err(_) => vec![],
                };
                history.first()
                    .and_then(|r| r.get("organized_at").and_then(|v| v.as_str()))
                    .map(|s| s.starts_with(&today))
                    .unwrap_or(false)
            };
            if has_record_today {
                continue;
            }

            // 先取路径再执行整理，不持锁跑完整流程（避免阻塞其他命令）
            let (config_dir, data_dir) = {
                if let Ok(state_guard) = state.lock() {
                    (state_guard.config_dir.clone(), state_guard.data_dir.clone())
                } else {
                    continue;
                }
            };

            let _ = crate::pipeline::perform_organize(&config_dir, &data_dir);
            last_trigger_date = Some(today);
        }
    }
}
