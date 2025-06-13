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
    <button 
      @click="updateEarthImage"
      :disabled="isLoading"
      class="bg-blue-500 text-white px-4 py-2 rounded"
    >
      {{ isLoading ? '加载中...' : '获取地球图像' }}
    </button>
    
    <div v-if="tiles.length > 0" class="flex items-center">
      <button 
        @click="setAsWallpaper"
        :disabled="isLoading"
        class="bg-green-500 text-white px-4 py-2 rounded"
      >
        设为壁纸
      </button>
      <div class="ml-2 flex items-center">
        <!-- 新增自动设置开关 -->
        <div class="mt-2 flex items-center">
          <input 
            type="checkbox"
            v-model="autoSetWallpaper"
            id="autoSetWallpaper"
            class="mr-2 h-4 w-4 text-blue-600"
          />
          <label for="autoSetWallpaper" class="text-gray-300">自动设置为壁纸</label>
        </div>
      </div>
    </div>

    <p class="mt-2 text-gray-600">{{ status }}</p>
    
    <div v-if="errorMessage" class="mt-2 text-red-500">
      {{ errorMessage }}
    </div>
    
    <div v-if="tiles.length > 0" class="mt-4 grid grid-cols-4">
      <div v-for="(tile, index) in tiles" :key="index" class="aspect-square overflow-hidden">
        <img :src="tile" alt="Satellite tile" class="w-full h-full block p-0 m-0 border-0">
      </div>
    </div>

    <button @click="openImageDir" class="bg-green-500 text-white px-4 py-2 rounded">打开图片保存位置</button>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import { path } from '@tauri-apps/api';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
// 使用浏览器原生Base64编码 API
const encodeBase64 = (data: Uint8Array): string => {
  return btoa(String.fromCharCode(...data));
}; 
import { ref } from 'vue';
import { useStore } from '../store';

const store = useStore();
const tiles = ref<string[]>([]);
const mergedImagePath = ref('');
const tilesDir = ref('');
const status = ref('');
const isLoading = ref(false);
const errorMessage = ref('');

// 新增响应式变量
const autoSetWallpaper = ref(false);

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
          const tilePath = await path.join(tilesDir.value, `tile_${col}_${row}.png`);
          const imageBytes = await readFile(tilePath, { });
          const base64Data = encodeBase64(imageBytes);
          tiles.value.push(`data:image/png;base64,${base64Data}`);
        }
      }
      
      // 从文件名解析UTC时间并转换为北京时间
      const fileName = await path.basename(mergedImagePath.value) || '';
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
    if (autoSetWallpaper.value) {
      setTimeout(() => setAsWallpaper(), 1000);
    }
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
    errorMessage.value = '请先获取地球图像';
    return;
  }

  isLoading.value = true;
  status.value = '正在设置为壁纸...';

  try {
    await invoke('set_wallpaper', { path: mergedImagePath.value });
    status.value = '壁纸设置成功！';
  } catch (error) {
    console.error('Failed to set wallpaper:', error);
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
  // 或者用 openPath 直接打开
  // await openPath(dir)
}
</script>