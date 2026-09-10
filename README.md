# Al-Sahil Garage

تطبيق Windows مكتبي (Offline) لإدارة أدوار الزبائن في كراج تغيير زيت السيارات.

التقنيات: React + TypeScript + Vite + Tailwind CSS + shadcn/ui + Tauri + SQLite.

## متطلبات التطوير

- Windows 10/11
- Node.js 20+
- Rust stable (`rustup`)
- Visual Studio Build Tools مع workload **Desktop development with C++**
- WebView2 (مثبت عادةً مع Windows 10)

## تشغيل المشروع للتطوير

```bash
npm install
npm run tauri:dev
```

يفتح التطبيق كنافذة Windows وليس في المتصفح. شاشة الانتظار تُنشأ تلقائياً على الشاشة الثانية إن وُجدت.

اختبارات منطق الأدوار:

```bash
npm test
cd src-tauri
cargo test
```

## قاعدة البيانات

SQLite تُنشأ تلقائياً عند أول تشغيل في مجلد بيانات التطبيق:

`%AppData%\com.alsahil.garage\garage.db`

لا حاجة لتثبيت SQLite أو تشغيل خادم.

## الطابعة

من الإعدادات اختر الطابعة الحرارية وحجم الورق 58mm أو 80mm ثم **اختبار الطباعة**.

الطباعة تتم عبر Windows GDI مباشرة دون نافذة طباعة المتصفح. النص العربي يعتمد على خط Segoe UI في النظام.

إذا فشلت الطباعة يبقى الدور صالحاً ويمكن **إعادة طباعة**.

## الشاشة الثانية

1. اجعل الشاشة الثانية Extended (توسيع) وليس Duplicate (تكرار).
2. شغّل التطبيق.
3. تُفتح شاشة الانتظار تلقائياً Fullscreen على الشاشة غير الرئيسية.
4. من الإعدادات يمكن اختيار الشاشة يدوياً واختبارها.

## التشغيل التلقائي مع Windows

الإعدادات ← تفعيل **تشغيل التطبيق تلقائياً مع Windows**.

## إنشاء Windows Installer

```bash
npm run tauri:build
```

الملف الناتج يكون عادةً داخل:

`src-tauri/target/release/bundle/nsis/`

مثل:

`Al-Sahil Garage_1.0.0_x64-setup.exe`

يمكن إعادة تسميته إلى `Al-Sahil-Garage-Setup.exe` للتوزيع.

بعد التثبيت يظهر البرنامج باسم **Al-Sahil Garage** مع اختصار سطح المكتب وقائمة ابدأ. المستخدم النهائي لا يحتاج Node.js أو Rust أو npm.

## إصدار Production

1. حدّث رقم الإصدار في `package.json` و `src-tauri/tauri.conf.json` و `src-tauri/Cargo.toml`.
2. شغّل `npm test` و `cargo test --manifest-path src-tauri/Cargo.toml`.
3. شغّل `npm run tauri:build`.
4. وزّع ملف الـ NSIS فقط.
