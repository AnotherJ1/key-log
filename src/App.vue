<script setup lang="ts">
import { RouterLink, RouterView, useRoute } from "vue-router";
import { Odometer, DataLine, Setting } from "@element-plus/icons-vue";

const route = useRoute();

const menuItems = [
  { path: "/", name: "首页", icon: Odometer },
  { path: "/history", name: "历史统计", icon: DataLine },
  { path: "/settings", name: "设置", icon: Setting }
];
</script>

<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <h1>KeyLog</h1>
        <span>键盘鼠标统计</span>
      </div>
      
      <nav class="sidebar-menu">
        <RouterLink
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="menu-item"
          :class="{ active: route.path === item.path }"
        >
          <component :is="item.icon" class="icon" />
          <span>{{ item.name }}</span>
        </RouterLink>
      </nav>
      
      <div class="sidebar-footer">
        <div class="status-indicator">
          <span class="status-dot active"></span>
          <span>正在记录</span>
        </div>
      </div>
    </aside>
    
    <!-- 主内容区 -->
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>