import os from 'os'
import path from 'path'
import { spawn, spawnSync } from 'child_process'

let tauriDriver

export const config = {
  runner: 'local',
  automationProtocol: 'webdriver',

  specs: ['./test/specs/**/*.js'],
  maxInstances: 1,

  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  capabilities: [
    {
      platformName: 'mac',
      browserName: 'tauri', // REQUIRED dummy value
      'appium:automationName': 'tauri',
      'tauri:options': {
        application: './src-tauri/target/release/tauri-app',
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
    spawnSync('yarn', ['tauri', 'build'], {
      stdio: 'inherit',
    })
  },

  beforeSession: async () => {
    console.log('🚀 Starting tauri-driver...')
    tauriDriver = spawn(
      path.join(os.homedir(), '.cargo/bin/tauri-driver'),
      [],
      { stdio: 'inherit' }
    )
    await new Promise((resolve) => setTimeout(resolve, 1000))
  },

  afterSession: () => {
    if (tauriDriver) {
      console.log('🧹 Stopping tauri-driver')
      tauriDriver.kill()
    }
  },
}
