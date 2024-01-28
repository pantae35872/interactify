import { createApp } from 'vue'
import App from './App.vue'
import './assets/global.css'
import router from './router';
import { VueCookieNext } from 'vue-cookie-next';

createApp(App).use(router).use(VueCookieNext).mount('#app')

