import DefaultTheme from 'vitepress/theme'
import { EnhanceAppContext } from 'vitepress'
import './custom.css'

export default {
  ...DefaultTheme,
  enhanceApp({ router }: EnhanceAppContext) {
    if (typeof window !== 'undefined') {
      // Intercept feature card link clicks that target static .html files
      // and force a full browser navigation instead of a Vue Router push
      window.addEventListener('click', (e) => {
        const anchor = (e.target as HTMLElement).closest('a')
        if (!anchor) return
        const href = anchor.getAttribute('href')
        if (href && href.endsWith('.html') && href.startsWith('/')) {
          e.preventDefault()
          e.stopImmediatePropagation()
          // Replace current history entry with '/' so the back button
          // always returns to the home page instead of '/frontend' etc.
          history.replaceState(null, '', '/')
          window.location.href = href
        }
      }, true) // capture phase so we run before the router
    }
  }
}

