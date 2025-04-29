import transformerDirectives from '@unocss/transformer-directives';
import presetWind4 from '@unocss/preset-wind4';
import { defineConfig } from 'unocss';

export default defineConfig({
  presets: [presetWind4()],
  transformers: [transformerDirectives()],
  theme: {
    fontSize: {
      xs: ['11px', { 'line-height': '1.2' }],
      sm: ['12px', { 'line-height': '1.2' }],
      base: ['13px', { 'line-height': '1.2' }],
      lg: ['15px', { 'line-height': '1.2' }],
      xl: ['18px', { 'line-height': '1.2' }],
    },
  },
});
