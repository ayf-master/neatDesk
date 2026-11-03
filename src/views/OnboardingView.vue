<template>
  <div class="onboarding">
    <div class="onboarding-card">
      <div class="step-indicator">
        <span v-for="i in totalSteps" :key="i" class="dot" :class="{ active: currentStep === i - 1 }"></span>
      </div>

      <div v-if="currentStep === 0" class="step-content">
        <div class="step-icon">
          <span class="nd-logo">ND</span>
        </div>
        <h2>{{ $t('onboarding.welcome') }}</h2>
        <p class="step-desc">{{ $t('onboarding.welcomeDesc') }}</p>
        <div class="features-preview">
          <div v-for="(f, i) in $tm('onboarding.features')" :key="i" class="feature-item">{{ f }}</div>
        </div>
      </div>

      <div v-if="currentStep === 1" class="step-content">
        <h2>{{ $t('onboarding.usage') }}</h2>
        <div class="usage-list">
          <div class="usage-item">
            <span class="usage-num">1</span>
            <div>
              <strong>{{ $t('onboarding.usage1') }}</strong>
              <p>{{ $t('onboarding.usage1Desc') }}</p>
            </div>
          </div>
          <div class="usage-item">
            <span class="usage-num">2</span>
            <div>
              <strong>{{ $t('onboarding.usage2') }}</strong>
              <p>{{ $t('onboarding.usage2Desc') }}</p>
            </div>
          </div>
          <div class="usage-item">
            <span class="usage-num">3</span>
            <div>
              <strong>{{ $t('onboarding.usage3') }}</strong>
              <p>{{ $t('onboarding.usage3Desc') }}</p>
            </div>
          </div>
        </div>
      </div>

      <div v-if="currentStep === 2" class="step-content">
        <h2>{{ $t('onboarding.tips') }}</h2>
        <div class="tips-list">
          <div class="tip-item"><span class="tip-bullet">•</span><span>{{ $t('onboarding.tip1') }}</span></div>
          <div class="tip-item"><span class="tip-bullet">•</span><span>{{ $t('onboarding.tip2') }}</span></div>
          <div class="tip-item"><span class="tip-bullet">•</span><span>{{ $t('onboarding.tip3') }}</span></div>
          <div class="tip-item"><span class="tip-bullet">•</span><span>{{ $t('onboarding.tip4') }}</span></div>
        </div>
      </div>

      <div class="step-actions">
        <button v-if="currentStep > 0" class="btn btn-secondary" @click="prevStep">{{ $t('onboarding.prev') }}</button>
        <span v-else></span>
        <button v-if="currentStep < totalSteps - 1" class="btn btn-primary" @click="nextStep">{{ $t('onboarding.next') }}</button>
        <button v-else class="btn btn-primary" @click="finish">{{ $t('onboarding.start') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { api } from "../lib/api";
import type { AppConfig } from "../lib/types";

const emit = defineEmits<{ complete: [] }>();

const totalSteps = 3;
const currentStep = ref(0);

function nextStep() { if (currentStep.value < totalSteps - 1) currentStep.value++; }
function prevStep() { if (currentStep.value > 0) currentStep.value--; }

async function finish() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    config.settings.onboarding_completed = true;
    await api.updateConfig(config);
  } catch { /* ignore */ }
  emit("complete");
}
</script>

<style scoped>
.onboarding {
  position: fixed; inset: 0;
  background-color: var(--bg-primary);
  display: flex; align-items: center; justify-content: center;
  z-index: 500;
}
.onboarding-card { width: 460px; max-width: 90vw; }
.step-indicator { display: flex; justify-content: center; gap: 8px; margin-bottom: 32px; }
.dot { width: 8px; height: 8px; border-radius: 50%; background-color: var(--border-color); transition: all 0.3s; }
.dot.active { background-color: var(--color-primary); width: 24px; border-radius: 4px; }
.step-content { text-align: center; margin-bottom: 32px; }
.step-icon { margin-bottom: 20px; }
.nd-logo { display: inline-flex; align-items: center; justify-content: center; width: 64px; height: 64px; border-radius: 16px; background-color: var(--color-primary); color: white; font-size: 24px; font-weight: 800; letter-spacing: -0.5px; }
h2 { font-size: 22px; font-weight: 700; margin-bottom: 8px; }
.step-desc { color: var(--text-secondary); font-size: 15px; margin-bottom: 24px; }
.features-preview { display: flex; flex-direction: column; gap: 8px; text-align: left; padding: 16px 20px; background-color: var(--bg-hover); border-radius: var(--radius-md); }
.feature-item { font-size: 14px; }
.usage-list { display: flex; flex-direction: column; gap: 14px; text-align: left; }
.usage-item { display: flex; gap: 14px; align-items: flex-start; }
.usage-num { display: flex; align-items: center; justify-content: center; width: 28px; height: 28px; border-radius: 50%; background-color: var(--color-primary); color: white; font-size: 14px; font-weight: 700; flex-shrink: 0; }
.usage-item strong { display: block; font-size: 14px; margin-bottom: 2px; }
.usage-item p { font-size: 13px; color: var(--text-secondary); margin: 0; }
.tips-list { display: flex; flex-direction: column; gap: 12px; text-align: left; padding: 16px 20px; background-color: var(--bg-hover); border-radius: var(--radius-md); }
.tip-item { display: flex; gap: 8px; font-size: 14px; line-height: 1.5; }
.tip-bullet { color: var(--color-primary); font-weight: 700; }
.step-actions { display: flex; justify-content: space-between; }
</style>
