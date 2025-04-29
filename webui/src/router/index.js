import HomeView from '@/views/HomeView.vue';
import SigninView from '@/views/SigninView.vue';
import { createRouter, createWebHistory } from 'vue-router';

const base = window.__BASE__ == '/AITER_BASE' || !window.__BASE__ ? '/' : window.__BASE__;

const router = createRouter({
  history: createWebHistory(base),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/signin',
      name: 'signin',
      component: SigninView,
      meta: { transition: 'fade' },
    },
    {
      path: '/:pathMatch(.*)',
      component: () => import('@/views/NotFoundView.vue'),
    },
  ],
});

export default router;
