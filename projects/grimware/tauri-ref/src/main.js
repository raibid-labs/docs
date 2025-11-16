import { invoke } from '@tauri-apps/api/core'

// Get platform information on load
window.addEventListener('DOMContentLoaded', async () => {
  try {
    const platformInfo = await invoke('get_platform_info')
    document.querySelector('#platform-text').textContent = platformInfo
  } catch (error) {
    console.error('Error getting platform info:', error)
    document.querySelector('#platform-text').textContent = 'Error loading platform info'
  }
})

// Handle greet form
document.querySelector('#greet-form').addEventListener('submit', async (e) => {
  e.preventDefault()
  const input = document.querySelector('#greet-input')
  const name = input.value.trim()

  if (!name) {
    document.querySelector('#greet-msg').textContent = 'Please enter your name!'
    return
  }

  try {
    const greeting = await invoke('greet', { name })
    document.querySelector('#greet-msg').textContent = greeting
  } catch (error) {
    console.error('Error invoking greet:', error)
    document.querySelector('#greet-msg').textContent = 'Error: Could not greet'
  }
})
