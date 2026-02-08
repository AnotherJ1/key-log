<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useStatsStore } from "../stores/stats";
// 图标已在 main.ts 全局注册
import * as echarts from "echarts";

const statsStore = useStatsStore();
const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;
let refreshInterval: number | null = null;

// 格式化数字
function formatNumber(num: number): string {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + "万";
  }
  return num.toLocaleString();
}

// 初始化图表
function initChart() {
  if (!chartRef.value) return;
  
  chart = echarts.init(chartRef.value, "dark");
  updateChart();
}

// 更新图表
function updateChart() {
  if (!chart || !statsStore.topKeys.length) return;

  const topKeys = statsStore.topKeys.slice(0, 10);
  
  chart.setOption({
    backgroundColor: "transparent",
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "shadow" }
    },
    grid: {
      left: "3%",
      right: "4%",
      bottom: "3%",
      top: "3%",
      containLabel: true
    },
    xAxis: {
      type: "value",
      axisLine: { lineStyle: { color: "#3f3f46" } },
      axisLabel: { color: "#a1a1aa" },
      splitLine: { lineStyle: { color: "#27272a" } }
    },
    yAxis: {
      type: "category",
      data: topKeys.map(k => k.key_name).reverse(),
      axisLine: { lineStyle: { color: "#3f3f46" } },
      axisLabel: { color: "#e4e4e7" }
    },
    series: [{
      type: "bar",
      data: topKeys.map(k => k.count).reverse(),
      itemStyle: {
        borderRadius: [0, 4, 4, 0],
        color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
          { offset: 0, color: "#409eff" },
          { offset: 1, color: "#a855f7" }
        ])
      },
      label: {
        show: true,
        position: "right",
        color: "#e4e4e7",
        formatter: "{c}"
      }
    }]
  });
}

// 监听窗口大小变化
function handleResize() {
  chart?.resize();
}

onMounted(async () => {
  await statsStore.init();
  initChart();
  
  // 定时刷新（每3秒）
  refreshInterval = window.setInterval(async () => {
    await statsStore.refreshTodayStats();
    updateChart();
  }, 3000);
  
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
  chart?.dispose();
  window.removeEventListener("resize", handleResize);
});
</script>

<template>
  <div class="dashboard">
    <!-- 页面标题 -->
    <header class="page-header">
      <h1 class="page-title">今日统计</h1>
      <p class="page-subtitle">
        {{ statsStore.todayStats?.date || new Date().toLocaleDateString("zh-CN") }}
        · 实时更新
      </p>
    </header>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <div class="card stat-card">
        <div class="card-title">
          <el-icon><Keyboard /></el-icon>
          按键次数
        </div>
        <div class="card-value primary">
          {{ formatNumber(statsStore.totalKeys) }}
        </div>
      </div>
      
      <div class="card stat-card">
        <div class="card-title">
          <el-icon><Pointer /></el-icon>
          鼠标点击
        </div>
        <div class="card-value success">
          {{ formatNumber(statsStore.totalClicks) }}
        </div>
      </div>
      
      <div class="card stat-card">
        <div class="card-title">
          <el-icon><Timer /></el-icon>
          最后更新
        </div>
        <div class="card-value time">
          {{ statsStore.lastUpdateTime?.toLocaleTimeString("zh-CN") || "--:--:--" }}
        </div>
      </div>
    </div>

    <!-- 鼠标点击统计 -->
    <div class="mouse-stats card">
      <div class="card-title">鼠标点击分布</div>
      <div class="mouse-buttons">
        <div 
          v-for="stat in statsStore.mouseStats" 
          :key="stat.button"
          class="mouse-button-stat"
        >
          <span class="button-name">
            {{ 
              stat.button === 'Left' ? '左键' : 
              stat.button === 'Right' ? '右键' : 
              stat.button === 'Middle' ? '中键' :
              stat.button === 'ScrollUp' ? '滚轮上' :
              stat.button === 'ScrollDown' ? '滚轮下' :
              stat.button 
            }}
          </span>
          <span class="button-count">{{ formatNumber(stat.count) }}</span>
        </div>
        <div v-if="!statsStore.mouseStats.length" class="no-data">
          暂无数据
        </div>
      </div>
    </div>

    <!-- 按键排行榜 -->
    <div class="card chart-card">
      <div class="card-title">按键排行榜 Top 10</div>
      <div ref="chartRef" class="chart-container"></div>
      <div v-if="!statsStore.topKeys.length" class="no-data">
        开始输入后将显示统计数据
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  max-width: 1200px;
}

.stat-card {
  text-align: center;
}

.card-value.time {
  font-size: 24px;
  color: var(--text-secondary);
}

.mouse-stats {
  margin-bottom: 24px;
}

.mouse-buttons {
  display: flex;
  gap: 32px;
  margin-top: 16px;
}

.mouse-button-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 32px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.button-name {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.button-count {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.chart-card {
  min-height: 400px;
}

.chart-container {
  width: 100%;
  height: 350px;
}

.no-data {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted);
  font-size: 14px;
}
</style>
