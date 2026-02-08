import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

/**
 * Purpose:
 * Keep Android build configuration consistent across toolchains.
 *
 * How:
 * 1) Read `android_platform_version` from `justfile`.
 * 2) Read `minSdk` from `arhiv-android/app/build.gradle`.
 * 3) Fail if values differ, because cargo-ndk platform and Gradle minSdk
 *    must match for Android builds to be valid.
 */
const repoRoot = process.cwd();

const justfilePath = path.join(repoRoot, 'justfile');
const gradlePath = path.join(repoRoot, 'arhiv-android', 'app', 'build.gradle');

const justfile = fs.readFileSync(justfilePath, 'utf8');
const gradle = fs.readFileSync(gradlePath, 'utf8');

const platformMatch = justfile.match(/^android_platform_version := "(\d+)"/m);
if (!platformMatch) {
  console.error('Failed to read android_platform_version from justfile');
  process.exit(1);
}

const minSdkMatch = gradle.match(/^\s*minSdk\s+(\d+)/m);
if (!minSdkMatch) {
  console.error('Failed to read minSdk from arhiv-android/app/build.gradle');
  process.exit(1);
}

const platformVersion = platformMatch[1];
const minSdk = minSdkMatch[1];

if (platformVersion !== minSdk) {
  console.error(
    [
      'Android config mismatch:',
      `  justfile android_platform_version: ${platformVersion}`,
      `  build.gradle minSdk: ${minSdk}`,
      'Keep these values equal.',
    ].join('\n'),
  );
  process.exit(1);
}

console.log(`Android config OK: platform/minSdk=${platformVersion}`);
