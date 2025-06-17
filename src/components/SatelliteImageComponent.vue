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
    <div class="flex flex-wrap items-center gap-3 mb-6">
      <button
        @click="updateEarthImage"
        :disabled="isLoading"
        class="h-10 px-6 rounded-lg font-semibold transition
               bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-60"
      >
        {{ isLoading ? '加载中...' : '获取地球图像' }}
      </button>
      <button
        @click="setAsWallpaper"
        :disabled="!mergedImagePath"
        class="h-10 px-6 rounded-lg font-semibold transition
               bg-green-600 hover:bg-green-700 text-white disabled:opacity-60"
      >
        设为壁纸
      </button>
      <label class="flex items-center h-10 px-4 rounded-lg font-semibold bg-yellow-300 text-gray-800 cursor-pointer">
        <input type="checkbox" v-model="autoSetWallpaperEnabled" class="mr-2 accent-yellow-500" />
        自动每30分钟抓取并设置壁纸
      </label>
    </div>

    <div class="mb-2 text-base font-medium text-gray-200">
      最新地球图像时间（本地时区）：<span class="font-mono">{{ latestImageLocalTime }}</span>
    </div>

    <p class="mt-2 text-gray-600">{{ status }}</p>
    
    <div v-if="errorMessage" class="mt-2 text-red-500">
      {{ errorMessage }}
    </div>
    
    <div v-if="previewImage" class="mt-4">
      <img v-if="previewImage" :src="previewImage" alt="最新地球图像预览" />
    </div>

    <button
      @click="openImageDir"
      class="h-10 px-6 rounded-lg font-semibold transition
             bg-emerald-600 hover:bg-emerald-700 text-white mb-4"
    >
      打开图片保存位置
    </button>

    <button @click="cleanOldImagesNow" class="h-10 px-6 rounded-lg bg-red-600 text-white">
      立即清理旧图片
    </button>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { readFile, readDir } from '@tauri-apps/plugin-fs';
import { join, basename } from '@tauri-apps/api/path';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
// 使用浏览器原生Base64编码 API
const encodeBase64 = (data: Uint8Array): string => {
  return btoa(String.fromCharCode(...data));
}; 
import { ref, onMounted, watch, onUnmounted, computed } from 'vue';
import { useStore } from '../store';
import { platform } from '@tauri-apps/plugin-os';
import { load } from '@tauri-apps/plugin-store';
import { listen } from '@tauri-apps/api/event';

const store = useStore();
const tiles = ref<string[]>([]);
const mergedImagePath = ref('');
const previewImage = ref('');
const tilesDir = ref('');
const status = ref('');
const isLoading = ref(false);
const errorMessage = ref('');
const latestImageName = ref('');

// 新增响应式变量
const autoSetWallpaperEnabled = ref(false);
let autoSetTimer: ReturnType<typeof setInterval> | null = null;
let storeAutoSetWallpaperEnabled: Awaited<ReturnType<typeof load>> | null = null;

const cleanTimer = ref<ReturnType<typeof setInterval> | null>(null);

onMounted(async () => {
  storeAutoSetWallpaperEnabled = await load('settings.json');
  // 读取持久化的开关状态
  const saved = await storeAutoSetWallpaperEnabled.get<boolean>('autoSetWallpaperEnabled');
  if (typeof saved === 'boolean') {
    autoSetWallpaperEnabled.value = saved;
  }
  findLatestImage();
  listen('toggle-auto-set-wallpaper', () => {
    autoSetWallpaperEnabled.value = !autoSetWallpaperEnabled.value;
  });

  // 每小时定时清理一次
  cleanTimer.value = setInterval(() => {
    invoke('clean_old_images')
      .then(() => console.log('定时清理完成'))
      .catch(e => console.error('定时清理失败', e));
  }, 60 * 60 * 1000); // 1小时
});

// 自动任务逻辑
watch(autoSetWallpaperEnabled, async (val) => {
  if (val) {
    // 立即执行一次
    updateEarthImage().then(() => setAsWallpaper());
    // 每30分钟执行一次
    autoSetTimer = setInterval(() => {
      updateEarthImage().then(() => setAsWallpaper());
    }, 30 * 60 * 1000);
  } else {
    if (autoSetTimer) {
      clearInterval(autoSetTimer);
      autoSetTimer = null;
    }
  }
  if (storeAutoSetWallpaperEnabled) {
    await storeAutoSetWallpaperEnabled.set('autoSetWallpaperEnabled', val);
    await storeAutoSetWallpaperEnabled.save();
  }
});

// 组件卸载时清理定时器
onUnmounted(() => {
  if (autoSetTimer) {
    clearInterval(autoSetTimer);
    autoSetTimer = null;
  }
  if (cleanTimer.value) {
    clearInterval(cleanTimer.value);
    cleanTimer.value = null;
  }
});

async function findLatestImage() {
  try {
    // 获取图片目录
    const dir = await invoke<string>('get_image_dir');
    tilesDir.value = dir;
    // 读取目录下所有文件
    const files = await readDir(dir);
    // 只筛选 _black.png 结尾的图片
    const imageFiles = files
      .filter(f => f.name && /^earth_\d{8}_\d{4}_black\.png$/.test(f.name))
      .sort((a, b) => (a.name! > b.name! ? 1 : -1));
    if (imageFiles.length > 0) {
      // 取最新一张
      const latest = imageFiles[imageFiles.length - 1];
      mergedImagePath.value = await join(dir, latest.name!);
      latestImageName.value = latest.name!;
      await updatePreviewImage(); // 预览用的就是带黑边的图片
      status.value = `已加载最新地球图像（带黑边）：${latest.name}`;
    } else {
      status.value = '暂无带黑边的地球图像，请先抓取';
    }
  } catch (e) {
    errorMessage.value = '加载最新图片失败';
  }
}

async function updateEarthImage() {
  isLoading.value = true;
  status.value = '正在获取最新地球卫星图像...';
  errorMessage.value = '';

  try {
      const result = await invoke('update_earth_image');
      const data = JSON.parse(result as string);
      tilesDir.value = data.tiles_dir;
      mergedImagePath.value = data.merged_image;
      
      // 加载所有16个瓦片
      tiles.value = [];
      for (let row = 0; row < 4; row++) {
        for (let col = 0; col < 4; col++) {
          const tilePath = await join(tilesDir.value, `tile_${col}_${row}.png`);
          const imageBytes = await readFile(tilePath, { });
          const base64Data = encodeBase64(imageBytes);
          tiles.value.push(`data:image/png;base64,${base64Data}`);
        }
      }
      
      // 从文件名解析UTC时间并转换为北京时间
      const fileName = await basename(mergedImagePath.value) || '';
      const timeMatch = fileName.match(/earth_(\d{8})_(\d{4})\.png/);
      if (timeMatch) {
        const [, dateStr, timeStr] = timeMatch;
        const utcTime = new Date(`${dateStr.substring(0,4)}-${dateStr.substring(4,6)}-${dateStr.substring(6,8)}T${timeStr.substring(0,2)}:${timeStr.substring(2,4)}:00Z`);
        // 转换为北京时间(UTC+8)
        const beijingTime = new Date(utcTime.getTime() + 8 * 60 * 60 * 1000);
        status.value = `地球图像更新成功 (${beijingTime.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })})`;
      } else {
        status.value = '地球图像更新成功！';
      }
    
    // 新增自动设置逻辑
    if (autoSetWallpaperEnabled.value) {
      setTimeout(() => setAsWallpaper(), 1000);
    }
    await findLatestImage();
    await updatePreviewImage();
  } catch (error) {
    console.error('Failed to update earth image:', error);
    errorMessage.value = error instanceof Error ? error.message : String(error);
    status.value = '获取图像失败';
  } finally {
    isLoading.value = false;
  }
}

async function setAsWallpaper() {
  if (!mergedImagePath.value) {
    errorMessage.value = '没有可用的地球图像';
    return;
  }
  isLoading.value = true;
  status.value = '正在裁剪并设置为壁纸...';
  try {
    const currentPlatform = platform();
    const croppedPath = await invoke<string>('crop_and_set_wallpaper', {
      imagePath: mergedImagePath.value,
      platform: currentPlatform
    });
    status.value = '壁纸设置成功！';
    console.log('壁纸已设置，裁剪后图片路径:', croppedPath);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    status.value = '设置壁纸失败';
  } finally {
    isLoading.value = false;
  }
}

async function openImageDir() {
  const dir = await invoke<string>('get_image_dir')
  // 推荐用 revealItemInDir，高亮目录
  await revealItemInDir(dir)
}

async function updatePreviewImage() {
  if (!mergedImagePath.value) return;
  try {
    const imageBytes = await readFile(mergedImagePath.value);
    const base64Data = uint8ToBase64(imageBytes);
    previewImage.value = `data:image/png;base64,${base64Data}`;
  } catch (e) {
    console.error('图片读取失败', mergedImagePath.value, e);
    previewImage.value = '';
  }
}

function uint8ToBase64(bytes: Uint8Array): string {
  const CHUNK_SIZE = 0x8000; // 32KB
  let binary = '';
  for (let i = 0; i < bytes.length; i += CHUNK_SIZE) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + CHUNK_SIZE) as any);
  }
  return window.btoa(binary);
}

const latestImageLocalTime = computed(() => {
  if (!latestImageName.value) return '';
  // 假设文件名格式为 earth_YYYYMMDD_HHMM_black.png
  const match = latestImageName.value.match(/earth_(\d{8})_(\d{4})/);
  if (!match) return '';
  const [, dateStr, timeStr] = match;
  // 构造 UTC 时间
  const utcTime = new Date(
    `${dateStr.substring(0, 4)}-${dateStr.substring(4, 6)}-${dateStr.substring(6, 8)}T${timeStr.substring(0, 2)}:${timeStr.substring(2, 4)}:00Z`
  );
  // 转为本地时间字符串
  return utcTime.toLocaleString();
});

// 可选：加一个按钮手动清理
async function cleanOldImagesNow() {
  try {
    await invoke('clean_old_images');
    alert('图片清理完成！');
  } catch (e) {
    alert('图片清理失败');
  }
}
</script>