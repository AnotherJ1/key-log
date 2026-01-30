<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { getDateStats, getDateRangeSummary, type DateStats, type DaySummary } from "../utils/api";
// 图标已在 main.ts 全局注册
import * as echarts from "echarts";

const selectedDate = ref(new Date());
const dateStats = ref<DateStats | null>(null);
const summaryData = ref<DaySummary[]>([]);
const isLoading = ref(false);
const trendChartRef = ref<HTMLDivElement | null>(null);
let trendChart: echarts.ECharts | null = null;

// 格式化日期为 YYYY-MM-DD
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

// 获取过去30天的日期范围
function getLast30Days(): { start: string; end: string } {
  const end = new Date();
  const start = new Date();
  start.setDate(start.getDate() - 29);
  return {
    start: formatDate(start),
    end: formatDate(end)
  };
}

// 加载指定日期的统计
async function loadDateStats() {
  try {
    isLoading.value = true;
    const date = formatDate(selectedDate.value);
    dateStats.value = await getDateStats(date);
  } catch (error) {
    console.error("Failed to load date stats:", error);
  } finally {
    isLoading.value = false;
  }
}

// 加载趋势数据
async function loadSummaryData() {
  try {
    const { start, end } = getLast30Days();
    summaryData.value = await getDateRangeSummary(start, end);
    updateTrendChart();
  } catch (error) {
    console.error("Failed to load summary data:", error);
  }
}

// 初始化趋势图表
function initTrendChart() {
  if (!trendChartRef.value) return;
  
  trendChart = echarts.init(trendChartRef.value, "dark");
  updateTrendChart();
}

// 更新趋势图表
function updateTrendChart() {
  if (!trendChart || !summaryData.value.length) return;

  const sortedData = [...summaryData.value].sort((a, b) => a.date.localeCompare(b.date));
  
  trendChart.setOption({
    backgroundColor: "transparent",
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "cross" }
    },
    legend: {
      data: ["按键次数", "鼠标点击"],
      textStyle: { color: "#a1a1aa" },
      top: 0
    },
    grid: {
      left: "3%",
      right: "4%",
      bottom: "3%",
      top: "40px",
      containLabel: true
    },
    xAxis: {
      type: "category",
      data: sortedData.map(d => d.date.slice(5)), // MM-DD 格式
      axisLine: { lineStyle: { color: "#3f3f46" } },
      axisLabel: { color: "#a1a1aa", rotate: 45 }
    },
    yAxis: {
      type: "value",
      axisLine: { lineStyle: { color: "#3f3f46" } },
      axisLabel: { color: "#a1a1aa" },
      splitLine: { lineStyle: { color: "#27272a" } }
    },
    series: [
      {
        name: "按键次数",
        type: "line",
        data: sortedData.map(d => d.total_keys),
        smooth: true,
        itemStyle: { color: "#409eff" },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: "rgba(64, 158, 255, 0.3)" },
            { offset: 1, color: "rgba(64, 158, 255, 0)" }
          ])
        }
      },
      {
        name: "鼠标点击",
        type: "line",
        data: sortedData.map(d => d.total_clicks),
        smooth: true,
        itemStyle: { color: "#10b981" },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: "rgba(16, 185, 129, 0.3)" },
            { offset: 1, color: "rgba(16, 185, 129, 0)" }
          ])
        }
      }
    ]
  });
}

// 监听日期变化
watch(selectedDate, () => {
  loadDateStats();
});

// 监听窗口大小
function handleResize() {
  trendChart?.resize();
}

onMounted(async () => {
  await Promise.all([loadDateStats(), loadSummaryData()]);
  initTrendChart();
  window.addEventListener("resize", handleResize);
});
</script>

<template>
  <div class="history-page">
    <!-- 页面标题 -->
    <header class="page-header">
      <h1 class="page-title">历史统计</h1>
      <p class="page-subtitle">查看每日按键和鼠标使用详情</p>
    </header>

    <!-- 日期选择和当日统计 -->
    <div class="date-section">
      <div class="card date-picker-card">
        <div class="card-title">
          <el-icon><Calendar /></el-icon>
          选择日期
        </div>
        <el-date-picker
          v-model="selectedDate"
          type="date"
          placeholder="选择日期"
          :disabled-date="(date: Date) => date > new Date()"
          format="YYYY 年 MM 月 DD 日"
          style="width: 100%; margin-top: 12px;"
        />
      </div>

      <div class="card day-summary-card" v-loading="isLoading">
        <div class="card-title">{{ formatDate(selectedDate) }} 统计</div>
        <div class="day-stats">
          <div class="day-stat">
            <span class="stat-label">按键次数</span>
            <span class="stat-value primary">{{ dateStats?.total_keys?.toLocaleString() || 0 }}</span>
          </div>
          <div class="day-stat">
            <span class="stat-label">鼠标点击</span>
            <span class="stat-value success">{{ dateStats?.total_clicks?.toLocaleString() || 0 }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 按键排行榜表格 -->
    <div class="card" v-loading="isLoading">
      <div class="card-title">按键排行榜</div>
      <el-table
        :data="dateStats?.key_stats || []"
        style="width: 100%"
        max-height="400"
        empty-text="当日无数据"
      >
        <el-table-column type="index" label="排名" width="80" />
        <el-table-column prop="key_name" label="按键" width="150" />
        <el-table-column prop="count" label="次数">
          <template #default="{ row }">
            <div class="count-cell">
              <span>{{ row.count.toLocaleString() }}</span>
              <div class="count-bar">
                <div 
                  class="count-bar-fill" 
                  :style="{ width: `${(row.count / (dateStats?.key_stats?.[0]?.count || 1)) * 100}%` }"
                ></div>
              </div>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 趋势图表 -->
    <div class="card trend-card">
      <div class="card-title">
        <el-icon><TrendCharts /></el-icon>
        过去 30 天趋势
      </div>
      <div ref="trendChartRef" class="trend-chart"></div>
      <div v-if="!summaryData.length" class="no-data">
        暂无历史数据
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-page {
  max-width: 1200px;
}

.date-section {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 20px;
  margin-bottom: 24px;
}

.day-stats {
  display: flex;
  gap: 48px;
  margin-top: 16px;
}

.day-stat {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
}

.stat-value.primary {
  color: var(--primary-color);
}

.stat-value.success {
  color: var(--success-color);
}

.count-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.count-bar {
  flex: 1;
  height: 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
  overflow: hidden;
}

.count-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), #a855f7);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.trend-card {
  margin-top: 24px;
  min-height: 350px;
}

.trend-chart {
  width: 100%;
  height: 300px;
}

.no-data {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted);
}

@media (max-width: 768px) {
  .date-section {
    grid-template-columns: 1fr;
  }
}
</style>
