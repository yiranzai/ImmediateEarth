<!--
 * @Author: yiranzai wuqingdzx@gmail.com
 * @Date: 2025-06-12 21:12:07
 * @LastEditors: yiranzai wuqingdzx@gmail.com
 * @LastEditTime: 2025-06-13 01:13:30
 * @FilePath: \ImmediateEarth\src\components\SatelliteImageComponent.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<template>
  <div class="satellite-image-container">
    <div class="mb-6 flex gap-2">
      <button
        :class="activeTab === 'main' ? 'bg-blue-600 text-white' : 'bg-gray-200 text-gray-700'"
        class="px-4 py-2 rounded-t font-semibold"
        @click="activeTab = 'main'"
      >
        主界面
      </button>
      <button
        :class="activeTab === 'settings' ? 'bg-blue-600 text-white' : 'bg-gray-200 text-gray-700'"
        class="px-4 py-2 rounded-t font-semibold"
        @click="activeTab = 'settings'"
      >
        设置
      </button>
          <!-- 新增：地球大图时间显示 -->
    <div v-if="latestImageLocalTime" class="px-4 py-2 rounded-t font-semibold">
      当前图片更新时间：<span class="text-yellow-500 font-bold">{{ latestImageLocalTime }}</span>
    </div>
    </div>

    <div v-show="activeTab === 'main'">
      <!-- 天气信息卡片 -->
      <div class="mb-6 bg-blue-300 text-blue-900 rounded shadow p-4">
        <div class="flex items-center gap-2 mt-2">
          <span>当前位置天气：</span>
          <span v-if="weatherInfo">{{ weatherInfo }}</span>
          <span v-else>加载中...</span>
        </div>
      </div>

      <!-- 新增：多屏缩略预览区 -->
      <div class="mb-4">
        <span class="block text-sm text-red-500 mb-1">屏幕布局预览：</span>
        <div class="bg-[#f3f4f6] p-5 inline-block">
          <canvas ref="previewCanvas" style="background: #f3f4f6"></canvas>
        </div>
      </div>

      <!-- 地球大图预览卡片 -->
      <div class="mb-8 bg-white rounded shadow p-4 flex flex-col items-center">
        <span class="block text-sm text-red-500 mb-1">地球大图预览：</span>
        <div v-if="previewImage" class="mt-4">
          <img v-if="previewImage" :src="previewImage" alt="最新地球图像预览" />
        </div>
      </div>
    </div>

    <div v-show="activeTab === 'settings'">
      <!-- 天气信息展示区 -->
      <div class="mb-4 p-3 rounded bg-blue-100 text-blue-900 font-semibold flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <span>OpenWeather Key：</span>
          <template v-if="!openWeatherKey">
            <input v-model="inputKey" type="text" class="border rounded px-2 py-1" placeholder="OpenWeather Key" />
            <button @click="saveKey" class="ml-2 px-3 py-1 bg-blue-500 text-white rounded">保存</button>
          </template>
          <template v-else>
            <span class="text-xs text-gray-500">已保存</span>
            <button @click="clearKey" class="ml-2 px-2 py-1 bg-gray-300 rounded text-gray-700">更换Key</button>
          </template>
        </div>
        <div class="flex items-center gap-2 mt-2">
          <span>城市：</span>
          <input v-model="cityInput" type="text" class="border rounded px-2 py-1" placeholder="请输入城市名" />
          <button @click="saveCity" class="ml-2 px-3 py-1 bg-blue-500 text-white rounded">保存</button>
          <button @click="autoGetCity" class="ml-2 px-3 py-1 bg-green-500 text-white rounded">自动获取城市</button>
          <span v-if="autoCityLoading" class="text-xs text-gray-500">自动获取中...</span>
        </div>
      </div>

      <!-- 自动定时设置壁纸开关 -->
      <div class="mb-6 bg-white rounded shadow p-4 flex items-center gap-3">
        <label class="flex items-center cursor-pointer select-none">
          <input type="checkbox" v-model="autoSetWallpaperEnabled" class="form-checkbox h-5 w-5 text-green-600" />
          <span class="ml-2 text-gray-800 font-medium">自动定时更新并设置壁纸（每30分钟）</span>
        </label>
        <span v-if="autoSetWallpaperEnabled" class="ml-2 text-green-600 text-sm">已开启</span>
        <span v-else class="ml-2 text-gray-400 text-sm">已关闭</span>
      </div>

      <!-- 操作按钮区 -->
      <div class="mb-6 bg-white rounded shadow p-4 flex flex-wrap items-center gap-3">
        <button
          @click="updateEarthImage"
          :disabled="isLoading"
          class="h-10 px-6 rounded-lg font-semibold transition bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-60"
        >
          {{ isLoading ? '加载中...' : '获取地球图像' }}
        </button>
        <button
          @click="setAsWallpaperForAllMonitors"
          :disabled="!mergedImagePath || monitorIndexes.length === 0"
          class="h-10 px-6 rounded-lg font-semibold transition bg-green-600 hover:bg-green-700 text-white disabled:opacity-60"
        >
          {{ dynamicWallpaperBtnText }}
        </button>
        <button
          @click="openImageDir"
          class="h-10 px-6 rounded-lg font-semibold transition bg-emerald-600 hover:bg-emerald-700 text-white"
        >
          打开图片保存位置
        </button>
        <button @click="cleanOldImagesNow" class="h-10 px-6 rounded-lg bg-red-600 text-white">立即清理旧图片</button>
      </div>

      <!-- 屏幕选择卡片 -->
      <div class="mb-8 bg-gray-700 rounded shadow p-4">
        <div class="text-lg font-bold mb-2">选择自动设置壁纸的屏幕</div>
        <div class="flex flex-wrap gap-4">
          <label
            v-for="(monitor, idx) in monitors"
            :key="idx"
            class="flex items-center gap-2 bg-black-300 rounded px-3 py-2"
          >
            <input type="checkbox" :value="idx" v-model="monitorIndexes" />
            <span>
              <span v-if="monitor.is_primary" class="text-red-600 font-bold">【主屏幕】</span>
              屏幕{{ idx + 1 }}
              <template v-if="monitor.name">（{{ monitor.name }}）</template>
              <span class="text-xs text-gray-300 ml-1">
                {{ monitor.size?.[0] || monitor.size?.width }}x{{ monitor.size?.[1] || monitor.size?.height }} 缩放:{{
                  monitor.scale_factor
                }}
              </span>
            </span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { readFile, readDir } from '@tauri-apps/plugin-fs'
import { join, basename } from '@tauri-apps/api/path'
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
// 使用浏览器原生Base64编码 API
const encodeBase64 = (data: Uint8Array): string => {
  return btoa(String.fromCharCode(...data))
}
import { ref, onMounted, watch, onUnmounted, computed, nextTick } from 'vue'
import { useStore } from '../store'
import { platform } from '@tauri-apps/plugin-os'
import { load } from '@tauri-apps/plugin-store'
import { listen } from '@tauri-apps/api/event'

const store = useStore()
const tiles = ref<string[]>([])
const mergedImagePath = ref('')
const previewImage = ref('')
const tilesDir = ref('')
const status = ref('')
const isLoading = ref(false)
const errorMessage = ref('')
const latestImageName = ref('')

// 新增响应式变量
const autoSetWallpaperEnabled = ref(false)
let autoSetTimer: ReturnType<typeof setInterval> | null = null
let storeAutoSetWallpaperEnabled: Awaited<ReturnType<typeof load>> | null = null

const cleanTimer = ref<ReturnType<typeof setInterval> | null>(null)

const weatherInfo = ref('')
const cityInput = ref('')
const savedCity = ref('')
const autoCityLoading = ref(false)
const openWeatherKey = ref('')
const inputKey = ref('')

// 新增：获取所有显示器信息
const monitors = ref<any[]>([])

// 新增：初始化完成标志
const isInitialized = ref(false)

// 新增：用于存储选中的屏幕索引
const monitorIndexes = ref<number[]>([])

const dynamicWallpaperBtnText = computed(() => {
  if (monitorIndexes.value.length === 0) return '请选择屏幕'
  if (monitorIndexes.value.length === monitors.value.length) return '为所有屏幕设置壁纸'
  if (monitorIndexes.value.length === 1) {
    const idx = monitorIndexes.value[0]
    if (monitors.value[idx]?.is_primary) return '为主屏幕设置壁纸'
    return '为1个屏幕设置壁纸'
  }
  return `为${monitorIndexes.value.length}个屏幕设置壁纸`
})

// 新增：多屏缩略预览相关
const previewCanvas = ref<HTMLCanvasElement | null>(null)
const maxCanvasWidth = 900
const maxCanvasHeight = 600
const canvasPadding = 20 // px

// 计算包围盒时，所有 size/position 都乘以 scale_factor
const screenBoundingBox = computed(() => {
  if (!monitors.value.length) return { minX: 0, minY: 0, maxX: 0, maxY: 0 }
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity
  for (const m of monitors.value) {
    const x = (m.position?.x ?? (Array.isArray(m.position) ? m.position[0] : 0)) * (m.scale_factor ?? 1)
    const y = (m.position?.y ?? (Array.isArray(m.position) ? m.position[1] : 0)) * (m.scale_factor ?? 1)
    const w = (m.size?.width ?? (Array.isArray(m.size) ? m.size[0] : 0)) * (m.scale_factor ?? 1)
    const h = (m.size?.height ?? (Array.isArray(m.size) ? m.size[1] : 0)) * (m.scale_factor ?? 1)
    minX = Math.min(minX, x)
    minY = Math.min(minY, y)
    maxX = Math.max(maxX, x + w)
    maxY = Math.max(maxY, y + h)
  }
  return { minX, minY, maxX, maxY }
})

// 动态计算缩放比例，宽高都考虑
function getDynamicPreviewScale() {
  const { minX, minY, maxX, maxY } = screenBoundingBox.value
  const logicalWidth = maxX - minX
  const logicalHeight = maxY - minY
  // 日志：输出包围盒逻辑宽高
  console.log('[缩略图] 包围盒逻辑宽度:', logicalWidth, '逻辑高度:', logicalHeight)
  if (logicalWidth === 0 || logicalHeight === 0) return 1
  const scaleW = maxCanvasWidth / logicalWidth
  const scaleH = maxCanvasHeight / logicalHeight
  const dynamicScale = Math.min(scaleW, scaleH, 1)
  // 日志：输出缩放比例
  console.log('[缩略图] scaleW:', scaleW, 'scaleH:', scaleH, 'dynamicScale:', dynamicScale)
  return dynamicScale
}

function drawScreenPreview() {
  nextTick(() => {
    if (!previewCanvas.value || !monitors.value.length) {
      console.log('预览画布或显示器信息未就绪，将在100ms后重试')
      setTimeout(drawScreenPreview, 100)
      return
    }

    try {
      // 1. 先计算所有屏幕的逻辑像素坐标和大小
      const screens = monitors.value.map(m => {
        const scale = m.scale_factor ?? 1
        return {
          x: (m.position?.x ?? (Array.isArray(m.position) ? m.position[0] : 0)) / scale,
          y: (m.position?.y ?? (Array.isArray(m.position) ? m.position[1] : 0)) / scale,
          w: (m.size?.width ?? (Array.isArray(m.size) ? m.size[0] : 0)) / scale,
          h: (m.size?.height ?? (Array.isArray(m.size) ? m.size[1] : 0)) / scale,
          is_primary: m.is_primary,
        }
      })

      // 2. 计算包围盒
      const minX = Math.min(...screens.map(s => s.x))
      const minY = Math.min(...screens.map(s => s.y))
      const maxX = Math.max(...screens.map(s => s.x + s.w))
      const maxY = Math.max(...screens.map(s => s.y + s.h))
      const logicalWidth = maxX - minX
      const logicalHeight = maxY - minY

      // 3. 计算缩放比例
      const scale = Math.min(maxCanvasWidth / logicalWidth, maxCanvasHeight / logicalHeight, 1)

      // 4. 设置画布宽高
      const canvas = previewCanvas.value
      canvas.width = logicalWidth * scale + canvasPadding * 2
      canvas.height = logicalHeight * scale + canvasPadding * 2
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      ctx.fillStyle = '#f8fafc'
      ctx.fillRect(0, 0, canvas.width, canvas.height)

      // 5. 绘制每个屏幕
      for (let i = 0; i < screens.length; i++) {
        const s = screens[i]
        const sx = (s.x - minX) * scale + canvasPadding
        const sy = (s.y - minY) * scale + canvasPadding
        const sw = s.w * scale
        const sh = s.h * scale

        const img = monitorImages.value[i]
        console.log(`屏幕${i} img:`, img, img?.complete, monitorWallpapers.value[i])
        if (img && img.complete) {
          ctx.drawImage(img, sx, sy, sw, sh)
        } else {
          ctx.fillStyle = '#e5e7eb'
          ctx.fillRect(sx, sy, sw, sh)
        }
        // 编号
        ctx.save()
        const text = `屏幕${i + 1}`
        ctx.font = 'bold 18px sans-serif'
        const textWidth = ctx.measureText(text).width
        ctx.globalAlpha = 0.5
        ctx.fillStyle = 'black'
        ctx.fillRect(sx + 8, sy + 12, textWidth + 16, 28)
        ctx.globalAlpha = 1.0

        // 2. 黑色描边
        ctx.lineWidth = 4
        ctx.strokeStyle = 'black'
        ctx.strokeText(text, sx + 16, sy + 32)

        // 3. 白色填充
        ctx.fillStyle = 'white'
        ctx.fillText(text, sx + 16, sy + 32)
        ctx.restore()
      }
    } catch (error) {
      console.error('绘制预览失败:', error)
      // 如果绘制失败，100ms后重试
      setTimeout(drawScreenPreview, 100)
    }
  })
}
// 监听 monitors/previewImage 变化自动重绘
watch([monitors, previewImage], drawScreenPreview)

onMounted(async () => {
  storeAutoSetWallpaperEnabled = await load('settings.json')
  // 读取持久化的开关状态
  const saved = await storeAutoSetWallpaperEnabled.get<boolean>('autoSetWallpaperEnabled')
  if (typeof saved === 'boolean') {
    autoSetWallpaperEnabled.value = saved
  }
  // 读取已选中的屏幕索引
  const savedIndexes = await storeAutoSetWallpaperEnabled.get<number[]>('autoWallpaperMonitorIndexes')
  if (Array.isArray(savedIndexes)) {
    monitorIndexes.value = savedIndexes
  }
  findLatestImage()
  listen('toggle-auto-set-wallpaper', () => {
    autoSetWallpaperEnabled.value = !autoSetWallpaperEnabled.value
  })

  // 每小时定时清理一次
  cleanTimer.value = setInterval(
    () => {
      invoke('clean_old_images')
        .then(() => console.log('定时清理完成'))
        .catch(e => console.error('定时清理失败', e))
    },
    60 * 60 * 1000
  ) // 1小时

  loadKeyAndCity()

  // 获取显示器信息
  try {
    const monitorsInfo = await invoke<string>('get_all_monitors')
    monitors.value = JSON.parse(monitorsInfo)
  } catch (e) {
    console.error('获取显示器信息失败:', e)
  }
})

// 修改：watch autoSetWallpaperEnabled
watch(autoSetWallpaperEnabled, async val => {
  // 先清理已有定时器，避免重复
  if (autoSetTimer) {
    clearInterval(autoSetTimer)
    autoSetTimer = null
  }
  if (val && isInitialized.value) {
    // 立即执行一次
    await updateEarthImage()
    await setAsWallpaperForAllMonitors()
    // 设置30分钟定时器
    autoSetTimer = setInterval(
      async () => {
        await updateEarthImage()
        await setAsWallpaperForAllMonitors()
      },
      30 * 60 * 1000
    ) // 30分钟
  }
  // 保存设置到 store
  if (storeAutoSetWallpaperEnabled) {
    await storeAutoSetWallpaperEnabled.set('autoSetWallpaperEnabled', val)
    await storeAutoSetWallpaperEnabled.save()
  }
})

// 监听 monitorIndexes 变化，持久化保存
watch(monitorIndexes, async val => {
  if (storeAutoSetWallpaperEnabled) {
    await storeAutoSetWallpaperEnabled.set('autoWallpaperMonitorIndexes', val)
    await storeAutoSetWallpaperEnabled.save()
  }
})

// 组件卸载时清理定时器
onUnmounted(() => {
  if (autoSetTimer) {
    clearInterval(autoSetTimer)
    autoSetTimer = null
  }
  if (cleanTimer.value) {
    clearInterval(cleanTimer.value)
    cleanTimer.value = null
  }
})

// 新增：地球大图本地时间
const latestImageLocalTime = computed(() => {
  if (!latestImageName.value) return ''
  // 假设文件名格式为 earth_YYYYMMDD_HHMM_black.png
  const match = latestImageName.value.match(/earth_(\d{8})_(\d{4})/)
  if (!match) return ''
  const [, dateStr, timeStr] = match
  // 构造 UTC 时间
  const utcTime = new Date(
    `${dateStr.substring(0, 4)}-${dateStr.substring(4, 6)}-${dateStr.substring(6, 8)}T${timeStr.substring(0, 2)}:${timeStr.substring(2, 4)}:00Z`
  )
  // 转为本地时间字符串
  return utcTime.toLocaleString()
})

async function findLatestImage() {
  try {
    // 获取图片目录
    const dir = await invoke<string>('get_image_dir')
    tilesDir.value = dir
    // 读取目录下所有文件
    const files = await readDir(dir)
    // 只筛选 _black.png 结尾的图片
    const imageFiles = files
      .filter(f => f.name && /^earth_\d{8}_\d{4}_black\.png$/.test(f.name))
      .sort((a, b) => (a.name! > b.name! ? 1 : -1))
    if (imageFiles.length > 0) {
      const latest = imageFiles[imageFiles.length - 1]
      mergedImagePath.value = await join(dir, latest.name!)
      latestImageName.value = latest.name! // 记录最新图片文件名
      await updatePreviewImage()
      status.value = `已加载最新地球图像（带黑边）：${latest.name}`
    } else {
      status.value = '暂无带黑边的地球图像，请先抓取'
    }
    // 检查初始化是否完成
    checkInitialization()
  } catch (e) {
    errorMessage.value = '加载最新图片失败'
  }
}

async function updateEarthImage() {
  isLoading.value = true
  status.value = '正在获取最新地球卫星图像...'
  errorMessage.value = ''

  try {
    const result = await invoke('update_earth_image')
    const data = JSON.parse(result as string)
    tilesDir.value = data.tiles_dir
    mergedImagePath.value = data.merged_image

    // 加载所有16个瓦片
    tiles.value = []
    for (let row = 0; row < 4; row++) {
      for (let col = 0; col < 4; col++) {
        const tilePath = await join(tilesDir.value, `tile_${col}_${row}.png`)
        const imageBytes = await readFile(tilePath, {})
        const base64Data = encodeBase64(imageBytes)
        tiles.value.push(`data:image/png;base64,${base64Data}`)
      }
    }

    // 从文件名解析UTC时间并转换为北京时间
    const fileName = (await basename(mergedImagePath.value)) || ''
    const timeMatch = fileName.match(/earth_(\d{8})_(\d{4})\.png/)
    if (timeMatch) {
      const [, dateStr, timeStr] = timeMatch
      const utcTime = new Date(
        `${dateStr.substring(0, 4)}-${dateStr.substring(4, 6)}-${dateStr.substring(6, 8)}T${timeStr.substring(0, 2)}:${timeStr.substring(2, 4)}:00Z`
      )
      // 转换为北京时间(UTC+8)
      const beijingTime = new Date(utcTime.getTime() + 8 * 60 * 60 * 1000)
      status.value = `地球图像更新成功 (${beijingTime.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })})`
    } else {
      status.value = '地球图像更新成功！'
    }

    await findLatestImage()
    await updatePreviewImage()
  } catch (error) {
    console.error('Failed to update earth image:', error)
    errorMessage.value = error instanceof Error ? error.message : String(error)
    status.value = '获取图像失败'
  } finally {
    isLoading.value = false
  }
}

async function setAsWallpaperForAllMonitors() {
  if (!mergedImagePath.value) {
    errorMessage.value = '没有可用的地球图像'
    return
  }

  isLoading.value = true
  status.value = '正在为所有屏幕设置壁纸...'

  try {
    const currentPlatform = await platform()
    const results = await invoke<string>('set_wallpaper_for_all_monitors', {
      imagePath: mergedImagePath.value,
      platform: currentPlatform,
      monitorIndexes: monitorIndexes.value,
    })

    const paths = JSON.parse(results)
    status.value = `壁纸设置成功！已为 ${paths.length} 个屏幕设置壁纸`
    console.log('壁纸已设置，裁剪后图片路径:', paths)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
    status.value = '设置壁纸失败'
  } finally {
    isLoading.value = false
  }
}

async function openImageDir() {
  const dir = await invoke<string>('get_image_dir')
  // 推荐用 revealItemInDir，高亮目录
  await revealItemInDir(dir)
}

async function updatePreviewImage() {
  if (!mergedImagePath.value) return
  try {
    const imageBytes = await readFile(mergedImagePath.value)
    const base64Data = uint8ToBase64(imageBytes)
    previewImage.value = `data:image/png;base64,${base64Data}`
    // 检查初始化是否完成
    checkInitialization()
  } catch (e) {
    console.error('图片读取失败', mergedImagePath.value, e)
    previewImage.value = ''
  }
}

function uint8ToBase64(bytes: Uint8Array): string {
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return window.btoa(binary)
}

// 可选：加一个按钮手动清理
async function cleanOldImagesNow() {
  try {
    await invoke('clean_old_images')
    alert('图片清理完成！')
  } catch (e) {
    alert('图片清理失败')
  }
}

// 读取本地 store 的 key 和城市
async function loadKeyAndCity() {
  const store = await load('settings.json')
  const key = await store.get<string>('openweather_key')
  if (key) openWeatherKey.value = key
  const city = await store.get<string>('weather_city')
  if (city) {
    savedCity.value = city
    cityInput.value = city
  }
}

// 保存 key 到 store
async function saveKey() {
  if (!inputKey.value) return
  const store = await load('settings.json')
  await store.set('openweather_key', inputKey.value)
  await store.save()
  openWeatherKey.value = inputKey.value
  inputKey.value = ''
  fetchWeather()
}

// 清除 key
async function clearKey() {
  const store = await load('settings.json')
  await store.delete('openweather_key')
  await store.save()
  openWeatherKey.value = ''
  weatherInfo.value = ''
}

// 保存城市到 store
async function saveCity() {
  if (!cityInput.value) return
  const store = await load('settings.json')
  await store.set('weather_city', cityInput.value)
  await store.save()
  savedCity.value = cityInput.value
  fetchWeather()
}

// 自动获取城市名
async function autoGetCity() {
  autoCityLoading.value = true
  try {
    const resp = await fetch('https://cip.cc')
    const text = await resp.text()
    // cip.cc 返回内容如：IP : 1.2.3.4\n地址 : 中国 广东 广州\n...
    // 尝试提取城市名（最后一个汉字词）
    const match = text.match(/地址\\s*:\\s*.+?([\\u4e00-\\u9fa5]{2,})\\s*$/m)
    if (match) {
      cityInput.value = match[1]
      await saveCity()
    } else {
      weatherInfo.value = '自动获取城市失败'
    }
  } catch (e) {
    weatherInfo.value = '自动获取城市失败'
  } finally {
    autoCityLoading.value = false
  }
}

// 获取天气
async function fetchWeather() {
  if (!savedCity.value || !openWeatherKey.value) {
    weatherInfo.value = ''
    return
  }
  weatherInfo.value = ''
  try {
    const result = await invoke<string>('get_weather', {
      city: savedCity.value,
      key: openWeatherKey.value,
    })
    weatherInfo.value = result
    // 检查初始化是否完成
    checkInitialization()
  } catch (e) {
    weatherInfo.value = '天气获取失败'
  }
}

// 监听 key 和城市变化自动刷新天气
watch([openWeatherKey, savedCity], ([key, city]) => {
  if (key && city) {
    fetchWeather()
  }
})

// 修改：checkInitialization
function checkInitialization() {
  // 当天气信息和预览图都加载完成时，标记为初始化完成
  if (weatherInfo.value && previewImage.value) {
    if (!isInitialized.value) {
      isInitialized.value = true
      // 只在首次初始化时，如果自动开关已开，立即执行一次
      if (autoSetWallpaperEnabled.value) {
        updateEarthImage().then(() => setAsWallpaperForAllMonitors())
      }
    }
  }
}

const monitorWallpapers = ref<string[]>([])
const monitorImages = ref<HTMLImageElement[]>([])

async function loadMonitorWallpapersAndDraw() {
  if (!monitors.value.length) return
  const baseDir = await invoke<string>('get_image_dir')
  const arr: string[] = []
  const imgArr: HTMLImageElement[] = []
  const loadPromises: Promise<void>[] = []

  for (let i = 0; i < monitors.value.length; i++) {
    const m = monitors.value[i]
    const w = m.size?.width ?? (Array.isArray(m.size) ? m.size[0] : 0)
    const h = m.size?.height ?? (Array.isArray(m.size) ? m.size[1] : 0)
    const imgPath = await join(baseDir, `monitor_${i}`, `wallpaper_${w}x${h}.png`)
    try {
      const bytes = await readFile(imgPath)
      const base64 = uint8ToBase64(bytes)
      const src = `data:image/png;base64,${base64}`
      arr[i] = src
      // 预加载图片，返回 Promise
      const img = new window.Image()
      imgArr[i] = img
      loadPromises.push(
        new Promise(resolve => {
          img.onload = () => resolve()
          img.onerror = () => resolve() // 即使加载失败也 resolve，避免卡死
          img.src = src
        })
      )
    } catch (e) {
      arr[i] = ''
      imgArr[i] = null
      loadPromises.push(Promise.resolve())
    }
  }
  monitorWallpapers.value = arr
  monitorImages.value = imgArr

  // 等所有图片加载完成后再绘制
  await Promise.all(loadPromises)
  drawScreenPreview()
}

// 监听 monitors 变化自动加载并绘制
watch(monitors, loadMonitorWallpapersAndDraw, { immediate: true })

const activeTab = ref('main') // 'main' 或 'settings'

// 监听activeTab变化，确保在切换tab时重新绘制预览
watch(activeTab, () => {
  nextTick(() => {
    drawScreenPreview()
  })
})
</script>
