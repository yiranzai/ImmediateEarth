<template>
  <div
    class="main-view relative w-screen h-screen bg-black flex items-center justify-center overflow-hidden"
    @dblclick="goToDetailsView"
    @contextmenu.prevent="showContextMenu"
  >
    <!-- Loading Overlay -->
    <div
      v-if="isLoading"
      class="absolute inset-0 bg-black bg-opacity-75 flex flex-col items-center justify-center z-10"
    >
      <div class="text-white text-2xl mb-4">正在拼合来自卫星的最新地球图像...</div>
      <div class="animate-spin rounded-full h-16 w-16 border-b-2 border-white"></div>
    </div>

    <img
      v-if="finalImageSrc"
      :src="finalImageSrc"
      alt="Latest Earth Image"
      class="final-image"
      :style="imageStyle"
      draggable="false"
    />
    <div v-else-if="!isLoading" class="text-white">加载地球图像失败，请检查网络连接。</div>
    <ClockOverlay />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useStore } from '../store'
import ClockOverlay from './ClockOverlay.vue'

const store = useStore()
const isLoading = ref(true)
const finalImageSrc = ref('')

const windowSize = ref({ width: window.innerWidth, height: window.innerHeight })
const japanHour = ref(0)
let timer: ReturnType<typeof setInterval> | null = null

const updateJapanHour = () => {
  const now = new Date()
  const utcHours = now.getUTCHours()
  japanHour.value = (utcHours + 9) % 24
}

const imageStyle = computed(() => {
  const screenW = windowSize.value.width
  const screenH = windowSize.value.height
  const imageSize = Math.max(screenW, screenH)
  const screenRatio = screenW / screenH

  let translateX = (screenW - imageSize) / 2
  let translateY = (screenH - imageSize) / 2

  if (screenRatio > 1) {
    translateY = 0
  } else {
    if (japanHour.value < 6) {
      translateX = screenW - imageSize
    } else {
      translateX = 0
    }
  }

  return {
    position: 'absolute' as const,
    width: `${imageSize}px`,
    height: `${imageSize}px`,
    transform: `translate(${translateX}px, ${translateY}px)`,
  }
})

const loadAndStitchImage = async () => {
  isLoading.value = true
  try {
    const now = new Date()
    now.setUTCMinutes(now.getUTCMinutes() - 30)
    const year = now.getUTCFullYear()
    const month = String(now.getUTCMonth() + 1).padStart(2, '0')
    const day = String(now.getUTCDate()).padStart(2, '0')
    const hour = String(now.getUTCHours()).padStart(2, '0')
    const minute = String(Math.floor(now.getUTCMinutes() / 10) * 10).padStart(2, '0')

    const multiple = 4
    const tileSize = 550
    const urls: string[] = []
    for (let y = 0; y < multiple; y++) {
      for (let x = 0; x < multiple; x++) {
        const url = `https://himawari.asia/img/D531106/${multiple}d/${tileSize}/${year}/${month}/${day}/${hour}${minute}00_${x}_${y}.png`
        urls.push(url)
      }
    }

    const imageBlobs = await Promise.all(
      urls.map(url => fetch(url).then(res => (res.ok ? res.blob() : Promise.reject(res.statusText)))),
    )
    const images = await Promise.all(
      imageBlobs.map(
        blob =>
          new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image()
            img.onload = () => resolve(img)
            img.onerror = reject
            img.src = URL.createObjectURL(blob)
          }),
      ),
    )

    const stitchedSize = tileSize * multiple
    const stitchCanvas = document.createElement('canvas')
    stitchCanvas.width = stitchedSize
    stitchCanvas.height = stitchedSize
    const stitchCtx = stitchCanvas.getContext('2d')!
    images.forEach((img, index) => {
      const x = (index % multiple) * tileSize
      const y = Math.floor(index / multiple) * tileSize
      stitchCtx.drawImage(img, x, y)
      URL.revokeObjectURL(img.src)
    })

    const border = Math.round(stitchedSize / 10)
    const finalSize = stitchedSize + border * 2
    const finalCanvas = document.createElement('canvas')
    finalCanvas.width = finalSize
    finalCanvas.height = finalSize
    const finalCtx = finalCanvas.getContext('2d')!
    finalCtx.fillStyle = 'black'
    finalCtx.fillRect(0, 0, finalSize, finalSize)
    finalCtx.drawImage(stitchCanvas, border, border)

    finalImageSrc.value = finalCanvas.toDataURL('image/png')
  } catch (error) {
    console.error('加载或拼接地球图像失败:', error)
    finalImageSrc.value = ''
  } finally {
    isLoading.value = false
  }
}

const onResize = () => {
  windowSize.value = { width: window.innerWidth, height: window.innerHeight }
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  updateJapanHour()
  timer = setInterval(updateJapanHour, 60 * 1000)
  loadAndStitchImage()
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  if (timer) {
    clearInterval(timer)
  }
})

const goToDetailsView = () => {
  store.setView('details')
}

const showContextMenu = async (event: MouseEvent) => {
  alert('图片"另存为"功能需在详情页操作。')
}
</script>

<style scoped>
.main-view {
  cursor: pointer;
}
.final-image {
  will-change: transform;
}
</style> 