# Localization Guide

*Last Updated: March 25, 2024*

This guide provides detailed instructions on implementing and maintaining localization in the PCI File Manager application.

## Overview

Localization (L10n) in PCI File Manager enables:
- Multiple language support for the user interface
- Region-specific formatting for dates, numbers, and currencies
- Culturally appropriate icons and graphics
- Right-to-Left (RTL) language support

## Architecture

The application uses the i18next framework integrated with React for localization:

```
src/
├── localization/
│   ├── i18n.js             # i18n configuration
│   ├── languages/          # Translation files
│   │   ├── en/             # English translations
│   │   │   ├── common.json
│   │   │   ├── errors.json
│   │   │   └── ...
│   │   ├── fr/             # French translations
│   │   │   ├── common.json
│   │   │   ├── errors.json
│   │   │   └── ...
│   │   └── ...
│   └── formatters/         # Custom formatters
│       ├── dateFormatter.js
│       ├── numberFormatter.js
│       └── ...
```

## Setting Up Localization

### 1. Installation and Setup

```bash
npm install i18next react-i18next i18next-http-backend i18next-browser-languagedetector
```

Configure i18next in `src/localization/i18n.js`:

```javascript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import Backend from 'i18next-http-backend';
import LanguageDetector from 'i18next-browser-languagedetector';

i18n
  .use(Backend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'en',
    debug: process.env.NODE_ENV === 'development',
    ns: ['common', 'errors', 'fileManager', 'settings'],
    defaultNS: 'common',
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: true,
    },
  });

export default i18n;
```

### 2. Translation Files Structure

Organize translation files by language and namespace:

**English (`src/localization/languages/en/common.json`):**
```json
{
  "app": {
    "name": "PCI File Manager",
    "tagline": "Secure file management for your business"
  },
  "navigation": {
    "home": "Home",
    "files": "Files",
    "uploads": "Uploads",
    "settings": "Settings"
  },
  "actions": {
    "upload": "Upload",
    "download": "Download",
    "delete": "Delete",
    "rename": "Rename",
    "share": "Share",
    "cancel": "Cancel",
    "confirm": "Confirm"
  }
}
```

**French (`src/localization/languages/fr/common.json`):**
```json
{
  "app": {
    "name": "PCI Gestionnaire de Fichiers",
    "tagline": "Gestion sécurisée des fichiers pour votre entreprise"
  },
  "navigation": {
    "home": "Accueil",
    "files": "Fichiers",
    "uploads": "Téléversements",
    "settings": "Paramètres"
  },
  "actions": {
    "upload": "Téléverser",
    "download": "Télécharger",
    "delete": "Supprimer",
    "rename": "Renommer",
    "share": "Partager",
    "cancel": "Annuler",
    "confirm": "Confirmer"
  }
}
```

## Using Translations in Code

### React Components

```jsx
import React from 'react';
import { useTranslation } from 'react-i18next';

function FileActionButtons({ file }) {
  const { t } = useTranslation('common');
  
  return (
    <div className="file-actions">
      <button className="download-btn">
        {t('actions.download')}
      </button>
      <button className="delete-btn">
        {t('actions.delete')}
      </button>
      <button className="share-btn">
        {t('actions.share')}
      </button>
    </div>
  );
}

export default FileActionButtons;
```

### With Variables and Pluralization

```jsx
import React from 'react';
import { useTranslation } from 'react-i18next';

function FileStatus({ count }) {
  const { t } = useTranslation('fileManager');
  
  return (
    <div className="file-status">
      {t('status.filesCount', { count })}
    </div>
  );
}

export default FileStatus;
```

In the translation file (`fileManager.json`):

```json
{
  "status": {
    "filesCount": "{{count}} file",
    "filesCount_plural": "{{count}} files"
  }
}
```

### Formatting Dates and Numbers

Create custom formatters:

```javascript
// src/localization/formatters/dateFormatter.js
import { format } from 'date-fns';
import { enUS, fr } from 'date-fns/locale';

const locales = {
  en: enUS,
  fr: fr,
};

export function formatDate(date, formatStr, language = 'en') {
  return format(new Date(date), formatStr, {
    locale: locales[language] || locales.en,
  });
}
```

Using the formatter:

```jsx
import React from 'react';
import { useTranslation } from 'react-i18next';
import { formatDate } from '../localization/formatters/dateFormatter';

function FileItem({ file }) {
  const { t, i18n } = useTranslation();
  const currentLanguage = i18n.language;
  
  return (
    <div className="file-item">
      <div className="file-name">{file.name}</div>
      <div className="file-date">
        {formatDate(file.createdAt, 'PPP', currentLanguage)}
      </div>
    </div>
  );
}

export default FileItem;
```

## RTL Language Support

### 1. Detecting RTL Languages

Create a utility function:

```javascript
// src/localization/utils/rtlDetector.js
const RTL_LANGUAGES = ['ar', 'he', 'fa', 'ur'];

export function isRTL(language) {
  return RTL_LANGUAGES.includes(language);
}
```

### 2. Applying RTL Styles

Use CSS logical properties and CSS variables:

```css
:root {
  --direction: ltr;
  --start: left;
  --end: right;
}

[dir="rtl"] {
  --direction: rtl;
  --start: right;
  --end: left;
}

.file-list-item {
  padding-inline-start: 1rem;
  margin-inline-end: 0.5rem;
  border-inline-start: 2px solid var(--border-color);
}
```

Apply RTL direction in your app:

```jsx
import React, { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { isRTL } from './localization/utils/rtlDetector';

function App() {
  const { i18n } = useTranslation();
  
  useEffect(() => {
    const dir = isRTL(i18n.language) ? 'rtl' : 'ltr';
    document.documentElement.setAttribute('dir', dir);
  }, [i18n.language]);
  
  return (
    <div className="app">
      {/* App content */}
    </div>
  );
}

export default App;
```

## Language Selection UI

Implement a language switcher component:

```jsx
import React from 'react';
import { useTranslation } from 'react-i18next';

const languages = [
  { code: 'en', name: 'English' },
  { code: 'fr', name: 'Français' },
  { code: 'es', name: 'Español' },
  { code: 'ar', name: 'العربية' }
];

function LanguageSwitcher() {
  const { i18n } = useTranslation();
  
  const changeLanguage = (lng) => {
    i18n.changeLanguage(lng);
    // Save preference to local storage
    localStorage.setItem('preferredLanguage', lng);
  };
  
  return (
    <div className="language-switcher">
      <select 
        value={i18n.language} 
        onChange={(e) => changeLanguage(e.target.value)}
      >
        {languages.map((lang) => (
          <option key={lang.code} value={lang.code}>
            {lang.name}
          </option>
        ))}
      </select>
    </div>
  );
}

export default LanguageSwitcher;
```

## Adding a New Language

To add a new language to the application:

1. Create a new folder in `src/localization/languages/` with the language code (e.g., `de` for German)
2. Copy all JSON files from the `en` folder to the new language folder
3. Translate all text in these JSON files to the new language
4. Add the new language to the language selector component
5. Test the application with the new language

## Best Practices

### 1. Translation Keys

- Use nested structures for organization
- Keep keys descriptive and context-aware
- Follow a consistent naming convention (camelCase)
- Group related keys under common parents

### 2. Handling Dynamic Content

- Avoid string concatenation in code
- Use interpolation for variables
- Add context notes for translators when needed

### 3. Maintenance

- Keep translation files synchronized across languages
- Document the meaning and usage of keys
- Consider using a translation management system (TMS)
- Involve native speakers in reviewing translations

### 4. Performance Considerations

- Load languages on demand
- Cache translations in local storage
- Consider bundling common translations with the app

## Testing Localization

### 1. Manual Testing

Test each supported language:
- Verify all text is translated
- Check formatting of dates, numbers, and currencies
- Verify layout works with varying text lengths
- Test RTL languages if supported

### 2. Automated Testing

Use unit tests to verify:
- Translation files are valid JSON
- All required keys exist in all language files
- No missing or extra keys between language files

Example testing script:

```javascript
// tests/localization.test.js
const fs = require('fs');
const path = require('path');

function loadTranslationFiles(langCode) {
  const langDir = path.join(__dirname, '../src/localization/languages', langCode);
  const files = fs.readdirSync(langDir);
  
  return files.reduce((acc, file) => {
    if (file.endsWith('.json')) {
      const namespace = file.replace('.json', '');
      acc[namespace] = JSON.parse(fs.readFileSync(path.join(langDir, file), 'utf8'));
    }
    return acc;
  }, {});
}

describe('Localization files', () => {
  const languages = ['en', 'fr', 'es', 'de'];
  const translations = {};
  
  beforeAll(() => {
    languages.forEach(lang => {
      translations[lang] = loadTranslationFiles(lang);
    });
  });
  
  test('All languages have the same namespaces', () => {
    const baseNamespaces = Object.keys(translations.en);
    
    languages.forEach(lang => {
      if (lang === 'en') return;
      
      const langNamespaces = Object.keys(translations[lang]);
      expect(langNamespaces.sort()).toEqual(baseNamespaces.sort());
    });
  });
  
  // Add more tests as needed
});
```

## Related Documentation

- [UI Component Guide](ui-component-guide.md)
- [Development Environment Setup Guide](development-environment-setup.md)
- [API Endpoints Reference](../api/endpoints-reference.md)

## External Resources

- [i18next Documentation](https://www.i18next.com/)
- [React Intl Documentation](https://formatjs.io/docs/react-intl/)
- [W3C Internationalization Guidelines](https://www.w3.org/International/techniques/authoring-html) 