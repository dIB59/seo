import os from 'os'
import path from 'path'
import process from 'node:process'
import { spawn, spawnSync } from 'child_process'

let tauriDriver

const isWindows = os.platform() === 'win32'
const binaryName = isWindows ? 'SEO Insikt crawler.exe' : 'SEO Insikt crawler'
const applicationPath = path.resolve(
  process.cwd(),
  'src-tauri',
  'target',
  'release',
  binaryName
)

export const config = {
  runner: 'local',

  specs: ['./test/specs/**/*.js'],
  maxInstances: 1,

  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  capabilities: [
    {
      platformName: isWindows ? 'windows' : os.platform() === 'darwin' ? 'mac' : 'linux',
      browserName: 'tauri', // Some drivers still expect this
      'tauri:options': {
        application: applicationPath,
      },
    },
  ],


  framework: 'mocha',
  reporters: ['spec'],

  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  onPrepare: () => {
    console.log('🔨 Building Tauri app...')
    const result = spawnSync('npm', ['run', 'tauri', 'build'], {
      stdio: 'inherit',
      shell: true,
    })
    if (result.status !== 0) {
      throw new Error('Tauri build failed')
    }
  },

  beforeSession: async () => {
    console.log('🚀 Starting tauri-driver...')
    const driverExecutable = isWindows ? 'tauri-driver.exe' : 'tauri-driver'
    const driverPath = path.join(os.homedir(), '.cargo/bin', driverExecutable)

    tauriDriver = spawn(driverPath, [], { stdio: 'inherit', shell: true })

    tauriDriver.on('error', (err) => {
      console.error('Failed to spawn tauri-driver:', err)
    })

    // Increase wait time for driver to start
    await new Promise((resolve) => setTimeout(resolve, 3000))
  },

  afterSession: () => {
    if (tauriDriver) {
      console.log('🧹 Stopping tauri-driver')
      tauriDriver.kill()
    }
  },
}

