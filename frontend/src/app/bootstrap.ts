import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import '../shared/ui/shell/app-layout.css'
import '../shared/ui/styles/index.css'
import '../style.css'
import '../styles/surfaces.css'
import '../styles/theme-classes.css'

export function mountClientApp(): void {
	const app = createApp(App)
	const pinia = createPinia()
	const queryClient = new QueryClient()

	app.use(pinia)
	app.use(VueQueryPlugin, { queryClient })
	app.mount('#app')
}
