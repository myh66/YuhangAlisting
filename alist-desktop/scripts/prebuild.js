import { createWriteStream } from "node:fs";
import { chmod, copyFile, mkdir, readdir, rm, stat } from "node:fs/promises";
import { get } from "node:https";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binariesDir = join(rootDir, "binaries");
const tempDir = join(binariesDir, ".download");
const force = process.argv.includes("--force");

const targets = {
  "win32-x64": {
    alist: {
      url: "https://github.com/AlistGo/alist/releases/latest/download/alist-windows-amd64.zip",
      archive: "alist-windows-amd64.zip",
      names: ["alist.exe"],
      output: "alist.exe",
    },
    rclone: {
      url: "https://downloads.rclone.org/rclone-current-windows-amd64.zip",
      archive: "rclone-windows-amd64.zip",
      names: ["rclone.exe"],
      output: "rclone.exe",
    },
  },
  "darwin-x64": {
    alist: {
      url: "https://github.com/AlistGo/alist/releases/latest/download/alist-darwin-amd64.tar.gz",
      archive: "alist-darwin-amd64.tar.gz",
      names: ["alist"],
      output: "alist",
    },
    rclone: {
      url: "https://downloads.rclone.org/rclone-current-osx-amd64.zip",
      archive: "rclone-osx-amd64.zip",
      names: ["rclone"],
      output: "rclone",
    },
  },
  "darwin-arm64": {
    alist: {
      url: "https://github.com/AlistGo/alist/releases/latest/download/alist-darwin-arm64.tar.gz",
      archive: "alist-darwin-arm64.tar.gz",
      names: ["alist"],
      output: "alist",
    },
    rclone: {
      url: "https://downloads.rclone.org/rclone-current-osx-arm64.zip",
      archive: "rclone-osx-arm64.zip",
      names: ["rclone"],
      output: "rclone",
    },
  },
  "linux-x64": {
    alist: {
      url: "https://github.com/AlistGo/alist/releases/latest/download/alist-linux-amd64.tar.gz",
      archive: "alist-linux-amd64.tar.gz",
      names: ["alist"],
      output: "alist",
    },
    rclone: {
      url: "https://downloads.rclone.org/rclone-current-linux-amd64.zip",
      archive: "rclone-linux-amd64.zip",
      names: ["rclone"],
      output: "rclone",
    },
  },
  "linux-arm64": {
    alist: {
      url: "https://github.com/AlistGo/alist/releases/latest/download/alist-linux-arm64.tar.gz",
      archive: "alist-linux-arm64.tar.gz",
      names: ["alist"],
      output: "alist",
    },
    rclone: {
      url: "https://downloads.rclone.org/rclone-current-linux-arm64.zip",
      archive: "rclone-linux-arm64.zip",
      names: ["rclone"],
      output: "rclone",
    },
  },
};

async function main() {
  const key = `${process.platform}-${process.arch}`;
  const target = targets[key];

  if (!target) {
    throw new Error(`Unsupported platform: ${key}`);
  }

  await mkdir(binariesDir, { recursive: true });
  await rm(tempDir, { recursive: true, force: true });
  await mkdir(tempDir, { recursive: true });

  await installBinary("AList", target.alist);
  await installBinary("Rclone", target.rclone);

  if (process.platform === "win32") {
    await installWinFspInstaller();
  }

  await rm(tempDir, { recursive: true, force: true });
  console.log("Sidecar binaries are ready.");
}

async function installBinary(label, asset) {
  const outputPath = join(binariesDir, asset.output);

  if (!force && (await exists(outputPath))) {
    console.log(`${label}: ${asset.output} already exists, skipping.`);
    return;
  }

  const archivePath = join(tempDir, asset.archive);
  const extractDir = join(tempDir, `${label.toLowerCase()}-extract`);

  console.log(`${label}: downloading ${asset.url}`);
  await download(asset.url, archivePath);
  await mkdir(extractDir, { recursive: true });
  await extractArchive(archivePath, extractDir);

  const executable = await findExecutable(extractDir, asset.names);
  if (!executable) {
    throw new Error(`${label}: executable not found in ${asset.archive}`);
  }

  await copyFile(executable, outputPath);

  if (process.platform !== "win32") {
    await chmod(outputPath, 0o755);
  }

  console.log(`${label}: installed ${asset.output}`);
}

async function installWinFspInstaller() {
  const outputPath = join(binariesDir, "winfsp.msi");

  if (!force && (await exists(outputPath))) {
    console.log("WinFsp: winfsp.msi already exists, skipping.");
    return;
  }

  console.log("WinFsp: resolving latest MSI from GitHub releases");
  const release = await downloadJson("https://api.github.com/repos/winfsp/winfsp/releases/latest");
  const asset = release.assets?.find((item) => /^winfsp-.*\.msi$/i.test(item.name));

  if (!asset?.browser_download_url) {
    throw new Error("WinFsp: MSI asset not found in latest release");
  }

  console.log(`WinFsp: downloading ${asset.browser_download_url}`);
  await download(asset.browser_download_url, outputPath);
  console.log("WinFsp: installed winfsp.msi");
}

function downloadJson(url) {
  return new Promise((resolveJson, rejectJson) => {
    const request = get(
      url,
      {
        headers: {
          "User-Agent": "alist-desktop-prebuild",
          Accept: "application/vnd.github+json",
        },
      },
      (response) => {
        if ([301, 302, 303, 307, 308].includes(response.statusCode ?? 0)) {
          if (!response.headers.location) {
            rejectJson(new Error(`Redirect response missing Location header: ${url}`));
            return;
          }

          const redirectedUrl = new URL(response.headers.location, url).toString();
          response.resume();
          downloadJson(redirectedUrl).then(resolveJson).catch(rejectJson);
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          rejectJson(new Error(`JSON request failed with HTTP ${response.statusCode}: ${url}`));
          return;
        }

        let body = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => {
          try {
            resolveJson(JSON.parse(body));
          } catch (error) {
            rejectJson(error);
          }
        });
      },
    );

    request.on("error", rejectJson);
  });
}

function download(url, destination) {
  return new Promise((resolveDownload, rejectDownload) => {
    const request = get(url, (response) => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode ?? 0)) {
        if (!response.headers.location) {
          rejectDownload(new Error(`Redirect response missing Location header: ${url}`));
          return;
        }

        const redirectedUrl = new URL(response.headers.location, url).toString();
        response.resume();
        download(redirectedUrl, destination)
          .then(resolveDownload)
          .catch(rejectDownload);
        return;
      }

      if (response.statusCode !== 200) {
        response.resume();
        rejectDownload(new Error(`Download failed with HTTP ${response.statusCode}: ${url}`));
        return;
      }

      const file = createWriteStream(destination);
      response.pipe(file);
      file.on("finish", () => file.close(resolveDownload));
      file.on("error", rejectDownload);
    });

    request.on("error", rejectDownload);
  });
}

async function extractArchive(archivePath, destination) {
  if (archivePath.endsWith(".tar.gz")) {
    await execFileAsync("tar", ["-xzf", archivePath, "-C", destination]);
    return;
  }

  if (archivePath.endsWith(".zip") && process.platform === "win32") {
    await execFileAsync("powershell.exe", [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      "Expand-Archive",
      "-LiteralPath",
      archivePath,
      "-DestinationPath",
      destination,
      "-Force",
    ]);
    return;
  }

  if (archivePath.endsWith(".zip")) {
    await execFileAsync("unzip", ["-oq", archivePath, "-d", destination]);
    return;
  }

  throw new Error(`Unsupported archive format: ${archivePath}`);
}

async function findExecutable(directory, names) {
  const entries = await readdir(directory);

  for (const entry of entries) {
    const fullPath = join(directory, entry);
    const metadata = await stat(fullPath);

    if (metadata.isDirectory()) {
      const nested = await findExecutable(fullPath, names);
      if (nested) {
        return nested;
      }
      continue;
    }

    if (names.includes(basename(fullPath))) {
      return fullPath;
    }
  }

  return null;
}

async function exists(filePath) {
  try {
    await stat(filePath);
    return true;
  } catch {
    return false;
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
