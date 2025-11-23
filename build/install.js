#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const GITHUB_RELEASES = 'https://api.github.com/repos/Saladin5101/QuickSky/releases/latest';
const BINARY_NAME = process.platform === 'win32' ? 'sky.exe' : 'sky';

function getPlatformInfo() {
    const platform = process.platform;
    const arch = process.arch;
    
    const platformMap = {
        'darwin': 'macos',
        'linux': 'linux', 
        'win32': 'windows'
    };
    
    const archMap = {
        'x64': 'amd64',
        'arm64': 'arm64'
    };
    
    return {
        platform: platformMap[platform] || platform,
        arch: archMap[arch] || arch
    };
}

function downloadBinary(url, dest) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(dest);
        https.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                // Follow redirect
                return downloadBinary(response.headers.location, dest);
            }
            
            if (response.statusCode !== 200) {
                reject(new Error(`Download failed: ${response.statusCode}`));
                return;
            }
            
            response.pipe(file);
            file.on('finish', () => {
                file.close();
                resolve();
            });
        }).on('error', reject);
    });
}

async function install() {
    try {
        console.log('📦 Installing QuickSky...');
        
        const { platform, arch } = getPlatformInfo();
        console.log(`🔍 Detected platform: ${platform}-${arch}`);
        
        // Create bin directory
        const binDir = path.join(__dirname, 'bin');
        if (!fs.existsSync(binDir)) {
            fs.mkdirSync(binDir, { recursive: true });
        }
        
        // Try to use local binary first (for development)
        const localBinary = path.join(__dirname, '..', 'target', 'release', BINARY_NAME);
        const targetBinary = path.join(binDir, BINARY_NAME);
        
        if (fs.existsSync(localBinary)) {
            console.log('📋 Using local binary...');
            fs.copyFileSync(localBinary, targetBinary);
        } else {
            console.log('🌐 Downloading binary from GitHub releases...');
            
            // Fetch latest release info
            const releaseData = await new Promise((resolve, reject) => {
                https.get(GITHUB_RELEASES, { headers: { 'User-Agent': 'quicksky-installer' } }, (res) => {
                    let data = '';
                    res.on('data', chunk => data += chunk);
                    res.on('end', () => resolve(JSON.parse(data)));
                }).on('error', reject);
            });
            
            // Find matching asset
            const assetName = `quicksky-${platform}-${arch}${platform === 'windows' ? '.exe' : ''}`;
            const asset = releaseData.assets.find(a => a.name === assetName);
            
            if (!asset) {
                throw new Error(`No binary found for ${platform}-${arch}`);
            }
            
            await downloadBinary(asset.browser_download_url, targetBinary);
        }
        
        // Make executable on Unix systems
        if (process.platform !== 'win32') {
            fs.chmodSync(targetBinary, '755');
        }
        
        console.log('✅ QuickSky installed successfully!');
        console.log('🚀 Run "sky --help" to get started');
        
    } catch (error) {
        console.error('❌ Installation failed:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    install();
}

module.exports = { install };