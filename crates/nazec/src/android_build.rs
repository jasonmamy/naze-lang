//! Android build: produces an Android project with WebView that runs Naze apps.

use std::fs;
use std::path::Path;

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;

/// Build an Android project with WebView.
pub fn run(manifest: &Manifest, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new(&manifest.build.output);
    let app_name = &manifest.app.name;

    // Step 1: Build app_data.bin using existing web build
    build::run(manifest, format)?;

    // Step 2: Create android project directory
    let android_dir = output_dir.join("android");
    let assets_dir = android_dir
        .join("app")
        .join("src")
        .join("main")
        .join("assets");
    fs::create_dir_all(&assets_dir)?;

    // Step 3: Copy app_data.bin
    let app_data_path = output_dir.join("app_data.bin");
    fs::copy(&app_data_path, assets_dir.join("app_data.bin"))?;

    // Step 4: Copy WASM runtime files (embedded in nazec binary)
    let runtime_js = include_bytes!("../../naze-runtime/pkg/naze_runtime.js");
    let runtime_wasm = include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");
    fs::write(assets_dir.join("naze_runtime.js"), runtime_js)?;
    fs::write(assets_dir.join("naze_runtime_bg.wasm"), runtime_wasm)?;

    // Step 5: Generate index.html for Android WebView
    let html = generate_android_html(app_name);
    fs::write(assets_dir.join("index.html"), html)?;

    // Step 6: Write Android project files
    write_android_project(&android_dir, app_name)?;

    if format == Format::Text {
        eprintln!("  created: {}/", android_dir.display());
        eprintln!();
        eprintln!("  to build APK:");
        eprintln!("    cd {}", android_dir.display());
        eprintln!("    gradle wrapper --gradle-version 8.2");
        eprintln!("    ./gradlew assembleDebug");
        eprintln!();
        eprintln!("  or open in Android Studio and build from there");
        eprintln!();
        eprintln!("  to install on device:");
        eprintln!("    adb install app/build/outputs/apk/debug/app-debug.apk");
    }

    Ok(())
}

fn generate_android_html(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no">
  <title>{title}</title>
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    html, body {{ width: 100%; height: 100%; overflow: hidden; background: #fff; }}
    canvas {{ display: block; touch-action: none; }}
  </style>
</head>
<body>
  <canvas id="naze-canvas"></canvas>
  <script type="module">
    import init, {{ start }} from './naze_runtime.js';

    async function main() {{
      await init();
      const resp = await fetch('./app_data.bin');
      const data = new Uint8Array(await resp.arrayBuffer());
      start(data, 'naze-canvas');
    }}

    main().catch(e => {{
      document.body.innerHTML = '<pre style="color:red;padding:20px">' + e + '</pre>';
    }});
  </script>
</body>
</html>
"#
    )
}

fn write_android_project(
    android_dir: &Path,
    app_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create directory structure
    let app_dir = android_dir.join("app");
    let main_dir = app_dir.join("src").join("main");
    let java_dir = main_dir.join("java").join("com").join("naze").join("app");
    let res_dir = main_dir.join("res").join("values");

    fs::create_dir_all(&java_dir)?;
    fs::create_dir_all(&res_dir)?;

    // Root build.gradle.kts
    fs::write(
        android_dir.join("build.gradle.kts"),
        r#"plugins {
    id("com.android.application") version "8.2.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.20" apply false
}
"#,
    )?;

    // settings.gradle.kts
    fs::write(
        android_dir.join("settings.gradle.kts"),
        format!(
            r#"pluginManagement {{
    repositories {{
        google()
        mavenCentral()
        gradlePluginPortal()
    }}
}}
dependencyResolutionManagement {{
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {{
        google()
        mavenCentral()
    }}
}}
rootProject.name = "{app_name}"
include(":app")
"#
        ),
    )?;

    // gradle.properties
    fs::write(
        android_dir.join("gradle.properties"),
        r#"org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
android.useAndroidX=true
kotlin.code.style=official
android.nonTransitiveRClass=true
"#,
    )?;

    // Note: Gradle wrapper files (gradlew, gradle-wrapper.jar) are not included.
    // Users can generate them with: gradle wrapper --gradle-version 8.2
    // Or open the project in Android Studio which handles this automatically.

    // App build.gradle.kts
    fs::write(
        app_dir.join("build.gradle.kts"),
        format!(
            r#"plugins {{
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}}

android {{
    namespace = "com.naze.app"
    compileSdk = 34

    defaultConfig {{
        applicationId = "com.naze.{app_name_safe}"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
    }}

    buildTypes {{
        release {{
            isMinifyEnabled = false
        }}
    }}
    compileOptions {{
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }}
    kotlinOptions {{
        jvmTarget = "1.8"
    }}
}}

dependencies {{
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.activity:activity-ktx:1.8.1")
    implementation("androidx.webkit:webkit:1.8.0")
}}
"#,
            app_name_safe = sanitize_package_name(app_name)
        ),
    )?;

    // AndroidManifest.xml
    fs::write(
        main_dir.join("AndroidManifest.xml"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <uses-permission android:name="android.permission.INTERNET" />

    <application
        android:allowBackup="true"
        android:label="@string/app_name"
        android:supportsRtl="true"
        android:theme="@style/Theme.NazeApp">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:configChanges="orientation|screenSize|keyboardHidden">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>

</manifest>
"#,
    )?;

    // MainActivity.kt
    fs::write(
        java_dir.join("MainActivity.kt"),
        r#"package com.naze.app

import android.os.Bundle
import android.view.View
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.ComponentActivity

class MainActivity : ComponentActivity() {
    private lateinit var webView: WebView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Hide system UI for fullscreen
        window.decorView.systemUiVisibility = (
            View.SYSTEM_UI_FLAG_FULLSCREEN
            or View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
            or View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
        )

        webView = WebView(this).apply {
            settings.apply {
                javaScriptEnabled = true
                domStorageEnabled = true
                allowFileAccess = true
                cacheMode = WebSettings.LOAD_NO_CACHE
            }
            webViewClient = WebViewClient()
            loadUrl("file:///android_asset/index.html")
        }
        setContentView(webView)
    }

    override fun onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
    }
}
"#,
    )?;

    // strings.xml
    fs::write(
        res_dir.join("strings.xml"),
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">{app_name}</string>
</resources>
"#
        ),
    )?;

    // themes.xml
    fs::write(
        res_dir.join("themes.xml"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="Theme.NazeApp" parent="android:Theme.Material.Light.NoActionBar">
        <item name="android:windowFullscreen">true</item>
        <item name="android:statusBarColor">@android:color/white</item>
        <item name="android:navigationBarColor">@android:color/white</item>
    </style>
</resources>
"#,
    )?;

    Ok(())
}

fn sanitize_package_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}
