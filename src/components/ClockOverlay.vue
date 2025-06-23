<template>
  <div class="clock-overlay text-white font-sans absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 text-center pointer-events-none">
    <div class="time text-7xl font-bold mb-2">
      {{ currentTime }}
    </div>
    <div class="date text-2xl">
      {{ currentDate }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const currentTime = ref('')
const currentDate = ref('')
let timer: ReturnType<typeof setInterval> | null = null

const updateDateTime = () => {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  currentDate.value = now.toLocaleDateString([], { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })
}

onMounted(() => {
  updateDateTime()
  timer = setInterval(updateDateTime, 1000)
})

onUnmounted(() => {
  if (timer) {
    clearInterval(timer)
  }
})
</script>

<style scoped>
.clock-overlay {
  text-shadow: 0 0 15px rgba(0, 0, 0, 0.5);
}
</style> 