import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { getTodayStats, getListeningStatus, type TodayStats } from "../utils/api";

export const useStatsStore = defineStore("stats", () => {
    // 状态
    const todayStats = ref<TodayStats | null>(null);
    const isListening = ref(false);
    const isLoading = ref(false);
    const lastUpdateTime = ref<Date | null>(null);
    const error = ref<string | null>(null);

    // 计算属性
    const totalKeys = computed(() => todayStats.value?.total_keys ?? 0);
    const totalClicks = computed(() => todayStats.value?.total_clicks ?? 0);
    const topKeys = computed(() => todayStats.value?.top_keys ?? []);
    const mouseStats = computed(() => todayStats.value?.mouse_stats ?? []);

    // 刷新今日统计
    async function refreshTodayStats() {
        try {
            isLoading.value = true;
            error.value = null;
            const stats = await getTodayStats();
            todayStats.value = stats;
            lastUpdateTime.value = new Date();
        } catch (e) {
            error.value = String(e);
            console.error("Failed to fetch today stats:", e);
        } finally {
            isLoading.value = false;
        }
    }

    // 刷新监听状态
    async function refreshListeningStatus() {
        try {
            isListening.value = await getListeningStatus();
        } catch (e) {
            console.error("Failed to get listening status:", e);
        }
    }

    // 初始化
    async function init() {
        await Promise.all([
            refreshTodayStats(),
            refreshListeningStatus()
        ]);
    }

    return {
        todayStats,
        isListening,
        isLoading,
        lastUpdateTime,
        error,
        totalKeys,
        totalClicks,
        topKeys,
        mouseStats,
        refreshTodayStats,
        refreshListeningStatus,
        init
    };
});
