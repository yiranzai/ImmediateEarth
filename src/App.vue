<!--
 * @Author: yiranzai wuqingdzx@gmail.com
 * @Date: 2025-06-12 20:19:33
 * @LastEditors: yiranzai wuqingdzx@gmail.com
 * @LastEditTime: 2025-06-12 23:25:00
 * @FilePath: \undefinedc:\Users\YiranzaiHWin\Documents\GitHub\ImmediateEarth\src\App.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<script setup lang="ts">
import { useStore } from './store'
const store = useStore()
store.initApp()
</script>

<template>
  <main class="bg-gray-900 min-h-screen">
    <header class="bg-gray-800 py-6 px-4 shadow-lg">
      <div class="max-w-4xl mx-auto">
        <h1 class="text-3xl font-bold text-white text-center">ImmediateEarth</h1>
        <p class="text-gray-300 text-center mt-2">实时向日葵8号卫星地球图像</p>
      </div>
    </header>
    <div class="max-w-6xl mx-auto p-4">
      <SatelliteImageComponent />
    </div>

    <footer class="bg-gray-800 text-gray-400 py-4 mt-8">
      <div class="max-w-4xl mx-auto text-center text-sm">
        <p>数据来源: 日本气象厅 向日葵8号卫星 (Himawari-8)</p>
        <p class="mt-1">图像每10分钟自动更新 | 涓流下载技术确保低带宽占用</p>
      </div>
    </footer>
  </main>
</template>
