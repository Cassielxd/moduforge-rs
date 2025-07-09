declare module 'vitepress-plugin-mermaid' {
  import { UserConfig } from 'vitepress'
  
  export interface MermaidConfig {
    theme?: string
    themeConfig?: {
      primaryColor?: string
      primaryTextColor?: string
      primaryBorderColor?: string
      lineColor?: string
      secondaryColor?: string
      tertiaryColor?: string
      background?: string
      mainBkg?: string
      secondBkg?: string
      tertiaryBkg?: string
    }
    flowchart?: {
      useMaxWidth?: boolean
      htmlLabels?: boolean
      curve?: string
    }
    sequence?: {
      useMaxWidth?: boolean
      wrap?: boolean
      width?: number
    }
    gantt?: {
      useMaxWidth?: boolean
      leftPadding?: number
      gridLineStartPadding?: number
    }
  }
  
  export interface VitePressWithMermaidConfig extends UserConfig {
    mermaid?: MermaidConfig
  }
  
  export function withMermaid<T extends UserConfig>(config: T): T & VitePressWithMermaidConfig
} 