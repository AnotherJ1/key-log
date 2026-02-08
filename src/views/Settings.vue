<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { 
  startListening, 
  stopListening, 
  getListeningStatus,
  isAutostartEnabled,
  enableAutostart,
  disableAutostart,
  getSetting,
  setSetting,
  openDataDir,
  getDbPath,
  setDbPath,
  exportToCsv,
  exportToExcel
} from "../utils/api";
import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
// @tauri-apps/plugin-opener 导出的函数名是 openUrl
import { openUrl } from '@tauri-apps/plugin-opener';
// 图标已在 main.ts 全局注册
import { ElMessage, ElMessageBox } from "element-plus";

const isListening = ref(false);
const autoStart = ref(false);
const minimizeToTray = ref(true);
const theme = ref("dark");
const isLoading = ref(false);
const isInitializing = ref(true);
const currentDbPath = ref("");
const exportDateRange = ref<[string, string]>([
  new Date().toISOString().split('T')[0],
  new Date().toISOString().split('T')[0]
]);
const isExporting = ref(false);
const isExportingExcel = ref(false);

const appVersion = "0.2.0";

// 打开数据目录
async function handleOpenDir() {
  try {
    await openDataDir();
  } catch (error) {
    ElMessage.error("无法打开目录: " + error);
  }
}

// 修改数据存储路径
async function handleChangeDbPath() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: "选择数据存储目录"
    });
    
    if (selected) {
      const newPath = `${selected}\\keylog.db`; // Windows path separator
      
      await ElMessageBox.confirm(
        `确定要将数据存储路径修改为：\n${newPath}\n\n修改后将自动创建新数据库或加载已有数据库。`,
        "修改存储位置",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning",
        }
      );
      
      await setDbPath(newPath);
      currentDbPath.value = newPath;
      ElMessage.success("数据库路径已更新，并已重新初始化连接");
      
      // 刷新状态以确保一切正常
      await refreshStatus();
    }
  } catch (error) {
    if (error !== "cancel") {
      ElMessage.error("修改失败: " + error);
    }
  }
}

// 导出 CSV
async function handleExport() {
  if (!exportDateRange.value || exportDateRange.value.length !== 2) {
    ElMessage.warning("请先选择导出日期范围");
    return;
  }

  const [startDate, endDate] = exportDateRange.value;

  try {
    const fileName = startDate === endDate 
      ? `keylog_stats_${startDate}.csv`
      : `keylog_stats_${startDate}_to_${endDate}.csv`;

    const filePath = await save({
      filters: [
        {
          name: "CSV 文件",
          extensions: ["csv"],
        },
      ],
      defaultPath: fileName,
    });

    if (filePath) {
      isExporting.value = true;
      await exportToCsv(startDate, endDate, filePath);
      ElMessage.success("导出成功: " + filePath);
    }
  } catch (error) {
    ElMessage.error("导出失败: " + error);
  } finally {
    isExporting.value = false;
  }
}

// 导出 Excel (分 Sheet)
async function handleExportExcel() {
  if (!exportDateRange.value || exportDateRange.value.length !== 2) {
    ElMessage.warning("请先选择导出日期范围");
    return;
  }

  const [startDate, endDate] = exportDateRange.value;

  try {
    const fileName = startDate === endDate 
      ? `keylog_stats_${startDate}.xlsx`
      : `keylog_stats_${startDate}_to_${endDate}.xlsx`;

    const filePath = await save({
      filters: [
        {
          name: "Excel 文件",
          extensions: ["xlsx"],
        },
      ],
      defaultPath: fileName,
    });

    if (filePath) {
      isExportingExcel.value = true;
      await exportToExcel(startDate, endDate, filePath);
      ElMessage.success("导出成功: " + filePath);
    }
  } catch (error) {
    ElMessage.error("导出失败: " + error);
  } finally {
    isExportingExcel.value = false;
  }
}

// 切换监听状态
async function toggleListening() {
  try {
    isLoading.value = true;
    if (isListening.value) {
      await stopListening();
      ElMessage.success("已停止记录");
    } else {
      await startListening();
      ElMessage.success("已开始记录");
    }
    isListening.value = !isListening.value;
  } catch (error) {
    ElMessage.error("操作失败: " + error);
  } finally {
    isLoading.value = false;
  }
}

// 获取监听状态和设置
async function refreshStatus() {
  try {
    isInitializing.value = true;
    
    // 获取监听状态
    isListening.value = await getListeningStatus();
    
    // 获取自启动状态
    autoStart.value = await isAutostartEnabled();
    
    // 获取最小化到托盘设置
    const traySetting = await getSetting("minimize_to_tray");
    if (traySetting !== null) {
      minimizeToTray.value = traySetting === "true";
    }
    
    // 获取主题设置
    const themeSetting = await getSetting("theme");
    if (themeSetting !== null) {
      theme.value = themeSetting;
    }

    // 获取当前数据库路径
    currentDbPath.value = await getDbPath();
    
  } catch (error) {
    console.error("Failed to get status:", error);
  } finally {
    isInitializing.value = false;
  }
}

// 监听自启动切换
watch(autoStart, async (newValue) => {
  if (isInitializing.value) return;
  
  try {
    if (newValue) {
      await enableAutostart();
      ElMessage.success("已开启开机自启动");
    } else {
      await disableAutostart();
      ElMessage.success("已关闭开机自启动");
    }
  } catch (error) {
    ElMessage.error("设置失败: " + error);
    // 恢复状态
    autoStart.value = !newValue;
  }
});

// 监听最小化到托盘切换
watch(minimizeToTray, async (newValue) => {
  if (isInitializing.value) return;
  
  try {
    await setSetting("minimize_to_tray", newValue.toString());
    ElMessage.success(newValue ? "已开启最小化到托盘" : "已关闭最小化到托盘");
  } catch (error) {
    ElMessage.error("保存设置失败: " + error);
  }
});

// 监听主题切换
watch(theme, async (newValue) => {
  if (isInitializing.value) return;
  
  try {
    await setSetting("theme", newValue);
    // 这里可以添加实际切换主题的代码
  } catch (error) {
    console.error("Failed to save theme setting:", error);
  }
});

// 打开外部链接
async function openExternalLink(url: string) {
  try {
    await openUrl(url);
  } catch (error) {
    ElMessage.error("无法打开链接: " + error);
  }
}

onMounted(() => {
  refreshStatus();
});
</script>

<template>
  <div class="settings-page">
    <!-- 页面标题 -->
    <header class="page-header">
      <h1 class="page-title">设置</h1>
      <p class="page-subtitle">应用配置和信息</p>
    </header>

    <!-- 记录控制 -->
    <div class="card settings-section">
      <h3 class="section-title">
        <el-icon><Setting /></el-icon>
        记录控制
      </h3>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">记录状态</span>
          <span class="setting-desc">控制键盘鼠标事件记录</span>
        </div>
        <div class="setting-control">
          <span :class="['status-badge', isListening ? 'active' : 'inactive']">
            {{ isListening ? '记录中' : '已停止' }}
          </span>
          <el-button 
            :type="isListening ? 'danger' : 'primary'"
            :loading="isLoading"
            @click="toggleListening"
          >
            {{ isListening ? '停止记录' : '开始记录' }}
          </el-button>
        </div>
      </div>
    </div>

    <!-- 启动设置 -->
    <div class="card settings-section">
      <h3 class="section-title">启动设置</h3>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">开机自启动</span>
          <span class="setting-desc">系统启动时自动运行 KeyLog</span>
        </div>
        <div class="setting-control">
          <el-switch v-model="autoStart" />
        </div>
      </div>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">最小化到托盘</span>
          <span class="setting-desc">关闭窗口时最小化到系统托盘</span>
        </div>
        <div class="setting-control">
          <el-switch v-model="minimizeToTray" />
        </div>
      </div>
    </div>

    <!-- 外观设置 -->
    <div class="card settings-section">
      <h3 class="section-title">外观设置</h3>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">主题模式</span>
          <span class="setting-desc">选择应用主题</span>
        </div>
        <div class="setting-control">
          <el-radio-group v-model="theme">
            <el-radio-button value="dark">深色</el-radio-button>
            <el-radio-button value="light" disabled>浅色</el-radio-button>
          </el-radio-group>
        </div>
      </div>
    </div>

    <!-- 数据管理 -->
    <div class="card settings-section">
      <h3 class="section-title">
        <el-icon><FolderOpened /></el-icon>
        数据管理
      </h3>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">数据存储位置</span>
          <span class="setting-desc">{{ currentDbPath || '正在加载...' }}</span>
        </div>
        <div class="setting-control">
          <el-button @click="handleChangeDbPath">修改位置</el-button>
          <el-button @click="handleOpenDir">打开目录</el-button>
        </div>
      </div>
      
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">导出数据</span>
          <span class="setting-desc">按日期范围导出统计数据</span>
        </div>
        <div class="setting-control export-control">
          <el-date-picker
            v-model="exportDateRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            value-format="YYYY-MM-DD"
            :clearable="false"
            size="small"
            style="width: 240px"
          />
          <div class="export-buttons">
            <el-button 
              type="primary" 
              :loading="isExporting"
              @click="handleExport"
            >
              导出 CSV (汇总)
            </el-button>
            <el-button 
              type="success" 
              :loading="isExportingExcel"
              @click="handleExportExcel"
            >
              导出 Excel (分日期)
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 关于 -->
    <div class="card settings-section">
      <h3 class="section-title">
        <el-icon><InfoFilled /></el-icon>
        关于
      </h3>
      
      <div class="about-info">
        <div class="app-logo">
          <h2>KeyLog</h2>
          <span class="version">v{{ appVersion }}</span>
        </div>
        <p class="about-desc">
          一款轻量级、跨平台的键盘鼠标使用统计工具。
          <br />
          使用 Tauri + Vue 3 构建，所有数据本地存储，保护您的隐私。
        </p>
        <div class="about-links">
          <el-link type="primary" :underline="false" @click="openExternalLink('https://github.com/AnotherJ1/key-log')">GitHub</el-link>
          <el-divider direction="vertical" />
          <el-link type="primary" :underline="false" @click="openExternalLink('https://github.com/AnotherJ1/key-log/issues')">问题反馈</el-link>
          <el-divider direction="vertical" />
          <el-link type="primary" :underline="false" @click="openExternalLink('https://github.com/AnotherJ1/key-log/blob/main/LICENSE')">MIT License</el-link>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 800px;
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.export-control {
  flex-direction: column;
  align-items: flex-end;
  gap: 12px;
}

.export-buttons {
  display: flex;
  gap: 8px;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-badge.active {
  background: rgba(16, 185, 129, 0.2);
  color: var(--success-color);
}

.status-badge.inactive {
  background: rgba(113, 113, 122, 0.2);
  color: var(--text-muted);
}

.about-info {
  text-align: center;
  padding: 24px 0;
}

.app-logo h2 {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, var(--primary-color), #a855f7);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 4px;
}

.version {
  font-size: 14px;
  color: var(--text-muted);
}

.about-desc {
  margin: 20px 0;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.about-links {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
}
</style>
