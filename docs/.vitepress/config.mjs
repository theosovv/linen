import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'en-US',
  title: 'Linen',
  description: 'A reactive JavaScript framework with fine-grained reactivity',
  
  lastUpdated: true,
  
  themeConfig: {
    logo: '/logo.svg',
    
    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'API', link: '/api/' },
      { text: 'Examples', link: '/examples/' },
      { text: 'GitHub', link: 'https://github.com/theosovv/linen' }
    ],
    
    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'What is Linen?', link: '/guide/' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Reactivity', link: '/guide/reactivity' }
          ]
        }
      ],
      '/api/': [
        {
          text: 'Core API',
          items: [
            { text: 'Overview', link: '/api/' },
            { text: 'signal()', link: '/api/signal' },
            { text: 'computed()', link: '/api/computed' },
            { text: 'effect()', link: '/api/effect' },
            { text: 'untrack()', link: '/api/untrack' },
            { text: 'events', link: '/api/events'},
            { text: 'batch()', link: '/api/batch' },
            { text: 'disposal', link: '/api/disposal' }
          ]
        },
        {
          text: 'DOM API',
          items: [
            { text: 'h', link: '/api/h' },
            { text: 'render', link: '/api/render' },
            { text: 'Fragment', link: '/api/fragment' },
          ]
        }
      ],
      '/examples/': [
        {
          text: 'Examples',
          items: [
            { text: 'Basic Counter', link: '/examples/' }
          ]
        }
      ]
    },
    
    socialLinks: [
      { icon: 'github', link: 'https://github.com/theosovv/linen' }
    ],
    
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2023-present Theosov'
    }
  }
})
