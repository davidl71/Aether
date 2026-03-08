import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';
import he from './locales/he.json';

const resources = {
  en: { translation: { common: en.common, app: en.app } },
  he: { translation: { common: he.common, app: he.app } },
};

i18n.use(initReactI18next).init({
  resources,
  lng: typeof navigator !== 'undefined' ? navigator.language.split('-')[0] : 'en',
  fallbackLng: 'en',
  supportedLngs: ['en', 'he'],
  defaultNS: 'translation',
  interpolation: {
    escapeValue: false,
  },
  react: {
    useSuspense: false,
  },
});

export default i18n;
