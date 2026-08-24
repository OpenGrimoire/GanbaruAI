import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const clientDir = path.resolve(scriptDir, "..");
const androidDir = path.join(clientDir, "src-tauri", "gen", "android");

/** Read one generated Android source file. */
async function readAndroidFile(relativePath) {
  const filePath = path.join(androidDir, relativePath);
  try {
    return await readFile(filePath, "utf8");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`read generated Android ${relativePath}: ${message}`);
  }
}

/** Read one generated Android binary resource. */
async function readAndroidBytes(relativePath) {
  const filePath = path.join(androidDir, relativePath);
  try {
    return await readFile(filePath);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`read generated Android ${relativePath}: ${message}`);
  }
}

/** Require a literal generated-project contract. */
function requireText(source, expected, label, failures) {
  if (!source.includes(expected)) failures.push(`${label} must contain ${JSON.stringify(expected)}`);
}

/** Reject a literal that would widen the generated-project contract. */
function rejectText(source, forbidden, label, failures) {
  if (source.includes(forbidden)) failures.push(`${label} must not contain ${JSON.stringify(forbidden)}`);
}

/** Count non-overlapping literal occurrences. */
function countText(source, value) {
  return source.split(value).length - 1;
}

const [
  rootBuild,
  appBuild,
  wrapper,
  manifest,
  mainActivity,
  dayTheme,
  nightTheme,
  filePaths,
  backupRules,
  extractionRules,
  launcherIcon,
] =
  await Promise.all([
    readAndroidFile("build.gradle.kts"),
    readAndroidFile("app/build.gradle.kts"),
    readAndroidFile("gradle/wrapper/gradle-wrapper.properties"),
    readAndroidFile("app/src/main/AndroidManifest.xml"),
    readAndroidFile("app/src/main/java/org/opengrimoire/ganbaruai/MainActivity.kt"),
    readAndroidFile("app/src/main/res/values/themes.xml"),
    readAndroidFile("app/src/main/res/values-night/themes.xml"),
    readAndroidFile("app/src/main/res/xml/file_paths.xml"),
    readAndroidFile("app/src/main/res/xml/backup_rules.xml"),
    readAndroidFile("app/src/main/res/xml/data_extraction_rules.xml"),
    readAndroidBytes("app/src/main/res/mipmap-xxxhdpi/ic_launcher.png"),
  ]);

const failures = [];

requireText(rootBuild, 'classpath("com.android.tools.build:gradle:8.11.0")', "root build", failures);
requireText(
  rootBuild,
  'classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:1.9.25")',
  "root build",
  failures,
);
requireText(wrapper, "gradle-8.14.3-bin.zip", "Gradle wrapper", failures);

for (const [expected, label] of [
  ['buildToolsVersion = "35.0.0"', "SDK Build Tools"],
  ["compileSdk = 36", "compile SDK"],
  ['ndkVersion = "30.0.15729638"', "NDK"],
  ["minSdk = 29", "minimum SDK"],
  ["targetSdk = 36", "target SDK"],
  ['applicationIdSuffix = ".dev"', "debug application ID suffix"],
  ['manifestPlaceholders["usesCleartextTraffic"] = "false"', "production cleartext policy"],
  ["sourceCompatibility = JavaVersion.VERSION_17", "Java source compatibility"],
  ["targetCompatibility = JavaVersion.VERSION_17", "Java target compatibility"],
  ['jvmTarget = "17"', "Kotlin JVM target"],
]) {
  requireText(appBuild, expected, label, failures);
}

requireText(manifest, '<uses-permission android:name="android.permission.INTERNET" />', "manifest", failures);
requireText(manifest, 'android:allowBackup="false"', "manifest", failures);
requireText(manifest, 'android:fullBackupContent="@xml/backup_rules"', "manifest", failures);
requireText(manifest, 'android:dataExtractionRules="@xml/data_extraction_rules"', "manifest", failures);
requireText(manifest, 'android:icon="@mipmap/ic_launcher"', "manifest", failures);
requireText(manifest, 'android:roundIcon="@mipmap/ic_launcher_round"', "manifest", failures);
rejectText(manifest, "LEANBACK_LAUNCHER", "manifest", failures);
rejectText(manifest, "android.software.leanback", "manifest", failures);
if (countText(manifest, "<uses-permission ") !== 1) {
  failures.push("manifest must declare exactly the INTERNET permission");
}

const launcherIconSha256 = createHash("sha256").update(launcherIcon).digest("hex");
if (launcherIconSha256 !== "158362b7787a594e4bb007281d89bd9f0575e5c583ddbfa9c9398621f4828055") {
  failures.push("xxxhdpi launcher icon must be regenerated from the Ganbaru AI icon manifest");
}

for (const expected of [
  "enableEdgeToEdge()",
  "WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()",
  "GanbaruAndroidInsets",
  "GanbaruAndroidAppearance",
  "ganbaru:android-insets",
  "@JavascriptInterface",
  "WebSettingsCompat.setAlgorithmicDarkeningAllowed(webView.settings, false)",
  "isAppearanceLightNavigationBars = lightTheme",
]) {
  requireText(mainActivity, expected, "Android system-bar inset bridge", failures);
}

for (const [theme, label] of [[dayTheme, "day theme"], [nightTheme, "night theme"]]) {
  requireText(theme, '<item name="android:forceDarkAllowed">false</item>', label, failures);
}

requireText(
  filePaths,
  '<external-files-path name="captured_images" path="Pictures/" />',
  "FileProvider paths",
  failures,
);
rejectText(filePaths, "<external-path", "FileProvider paths", failures);
rejectText(filePaths, 'path="."', "FileProvider paths", failures);

const backupDomains = [
  "root",
  "file",
  "database",
  "sharedpref",
  "external",
  "device_root",
  "device_file",
  "device_database",
  "device_sharedpref",
];
for (const domain of backupDomains) {
  const exclusion = `<exclude domain="${domain}" path="." />`;
  if (countText(backupRules, exclusion) !== 1) {
    failures.push(`legacy backup rules must exclude ${domain} exactly once`);
  }
  if (countText(extractionRules, exclusion) !== 2) {
    failures.push(`data extraction rules must exclude ${domain} from cloud and device transfer`);
  }
}

if (failures.length > 0) {
  console.error("Android generated-project contract failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exitCode = 1;
} else {
  console.log("Android generated-project contract passed.");
}
