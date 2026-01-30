import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
    {
        path: "/",
        name: "Dashboard",
        component: () => import("../views/Dashboard.vue"),
        meta: { title: "首页" }
    },
    {
        path: "/history",
        name: "History",
        component: () => import("../views/History.vue"),
        meta: { title: "历史统计" }
    },
    {
        path: "/settings",
        name: "Settings",
        component: () => import("../views/Settings.vue"),
        meta: { title: "设置" }
    }
];

const router = createRouter({
    history: createWebHashHistory(),
    routes
});

export default router;
