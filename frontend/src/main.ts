import '../themes/default.css'
import App from './app'

window.__besedka = new App()
window.addEventListener('DOMContentLoaded', () => window.__besedka.run())
