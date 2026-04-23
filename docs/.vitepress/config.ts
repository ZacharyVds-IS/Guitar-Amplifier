import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  base: '/Guitar-Amplifier/',
  title: "RustRiff documentation",
  description: "Documentation combining both frontend and backend docs for RustRiff",
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Frontend Docs', link: '/frontend/index.html' },
      { text: 'Backend Docs', link: '/backend/doc/rustriff_lib/index.html' }
    ],

    sidebar: [],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/ZacharyVds-IS/Guitar-Amplifier' }
    ]
  }
})
