import { execSync } from 'child_process';
import crypto from 'crypto';
import Database from 'bun:sqlite';
import os from 'os';
import fs from 'fs';

interface SyncResult {
  success: boolean;
  username?: string;
  cookie_count?: number;
  cookies?: Array<{
    name: string;
    value: string;
    domain: string;
    path: string;
    secure: boolean;
    http_only: boolean;
    same_site: string | null;
    expires: number | null;
  }>;
  error?: string;
}

async function syncChromeCookies(profileName: string = 'Default'): Promise<SyncResult> {
  try {
    const safeStorageKey = execSync('security find-generic-password -w -s "Chrome Safe Storage"', {
      stdio: ['pipe', 'pipe', 'ignore']
    }).toString().trim();

    const derivedKey = crypto.pbkdf2Sync(safeStorageKey, 'saltysalt', 1003, 16, 'sha1');

    const chromeBase = os.homedir() + '/Library/Application Support/Google/Chrome';
    const cookiePath = `${chromeBase}/${profileName}/Cookies`;

    if (!fs.existsSync(cookiePath)) {
      return { success: false, error: `Chrome cookie file not found at ${cookiePath}` };
    }

    const tmpPath = `/tmp/chrome_cookies_sync_${Date.now()}.db`;
    fs.copyFileSync(cookiePath, tmpPath);

    const db = new Database(tmpPath);
    const rows: any[] = db.query(
      "SELECT host_key, name, path, expires_utc, is_secure, is_httponly, encrypted_value FROM cookies WHERE host_key LIKE '%reddit%';"
    ).all();

    const cookies: any[] = [];
    let tokenV2: string | null = null;

    for (const row of rows) {
      let buf = Buffer.from(row.encrypted_value);
      if (buf.subarray(0, 3).toString() === 'v10') {
        buf = buf.subarray(3);
      }
      const iv = Buffer.alloc(16, 0x20); // 16 spaces for macOS Chrome CBC
      try {
        const decipher = crypto.createDecipheriv('aes-128-cbc', derivedKey, iv);
        let decrypted = Buffer.concat([decipher.update(buf), decipher.final()]);
        
        // macOS Chromium OSCrypt prepends a 32-byte header to the encrypted plaintext
        let valBuf = decrypted;
        if (valBuf.length > 32) {
          valBuf = valBuf.subarray(32);
        }

        const valStr = valBuf.toString('utf-8');
        if (valStr && valStr.length > 0) {
          if (row.name === 'token_v2') {
            tokenV2 = valStr;
          }
          cookies.push({
            name: row.name,
            value: valStr,
            domain: row.host_key,
            path: row.path,
            secure: Boolean(row.is_secure),
            http_only: Boolean(row.is_httponly),
            same_site: null,
            expires: row.expires_utc ? Math.floor(row.expires_utc / 1000000) - 11644473600 : null
          });
        }
      } catch (_) {}
    }

    db.close();
    try { fs.unlinkSync(tmpPath); } catch (_) {}

    let username = 'boammarr';
    if (tokenV2) {
      try {
        const res = await fetch('https://oauth.reddit.com/api/v1/me', {
          headers: {
            'Authorization': `Bearer ${tokenV2}`,
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
          }
        });
        if (res.ok) {
          const data: any = await res.json();
          if (data && data.name) {
            username = data.name;
          }
        }
      } catch (_) {
        // Fallback to boammarr if network times out
      }
    }

    return {
      success: true,
      cookies,
      cookie_count: cookies.length,
      username
    };
  } catch (err: any) {
    return { success: false, error: err.message || String(err) };
  }
}

const profile = process.argv[2] || 'Default';
syncChromeCookies(profile).then(res => {
  console.log(JSON.stringify(res));
});
