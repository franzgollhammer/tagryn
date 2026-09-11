import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './style.css';
import { api, native } from './api';

const app = createApp(App);
if (native) {
  const report = (error: unknown) => {
    void api('frontend_diagnostic', { message: String(error) }).catch(() => {});
  };
  window.addEventListener('error', (event) =>
    report(event.error || event.message),
  );
  window.addEventListener('unhandledrejection', (event) =>
    report(event.reason),
  );
  app.config.errorHandler = (error, _instance, info) => {
    report(`${String(error)} (${info})`);
    console.error(error);
  };
}
app.use(createPinia()).mount('#app');
