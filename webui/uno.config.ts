import presetWind from '@unocss/preset-wind';
import transformerDirectives from '@unocss/transformer-directives';
import { defineConfig } from 'unocss';

export default defineConfig({
  presets: [presetWind()],
  transformers: [transformerDirectives()],
  theme: {
    fontSize: {
      xs: ['12px', { 'line-height': '1.2' }],
      sm: ['13px', { 'line-height': '1.2' }],
      base: ['14px', { 'line-height': '1.2' }],
      lg: ['16px', { 'line-height': '1.2' }],
      xl: ['20px', { 'line-height': '1.2' }],
    },
  },
});
