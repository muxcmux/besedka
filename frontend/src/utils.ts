const HOST = "http://besedka.com:6353"

export function safeParse(text: string | undefined, def = {}) {
  try {
    return JSON.parse(text || '{}')
  } catch {
    return def
  }
}

export function message(msg: string, klass = 'error', element?: HTMLDivElement) {
  if (!element) element = document.getElementById('besedka-message') as HTMLDivElement
  element.innerHTML = `<div class="besedka-${klass}">${msg}</div>`
}

export function clearMessage(element?: HTMLDivElement) {
  document.getElementById('besedka-message')!.innerHTML = ''
  if (element) element.innerHTML = ''
}

export function setToken(token: string) {
  window.localStorage.setItem('__besedka_token', token)
}

export function getToken(): string | null {
  return window.localStorage.getItem('__besedka_token')
}

export function createElement<T extends HTMLElement>(el: string, className?: string, attributes?: {}): T {
  const element = document.createElement(el) as T
  if (className) element.className = className.split(' ').map(c => `besedka-${c}`).join(' ')
  for (const [key, value] of Object.entries(attributes || {})) {
    element.setAttribute(key, value as string)
  }
  return element
}

export function createButton(text: string, className?: string): HTMLButtonElement {
  const button = createElement<HTMLButtonElement>('button', className)
  button.textContent = text
  return button
}

export async function request<T>(endpoint: string, data: {}, method?: string, errorTarget?: HTMLDivElement): Promise<{ status: number, text: string, json: T | null }> {
  const body = JSON.stringify(data, replacer)
  try {
    const response = await fetch(`${HOST}${endpoint}`, {
      body,
      method: method || "POST",
      mode: "cors",
      headers: { "Content-Type": "application/json" }
    })

    const status = response.status
    let text = await response.text()
    let json: T | null = null

    if (response.status > 399 && response.status != 404) {
      message(text, 'error', errorTarget)
    } else {
      clearMessage(errorTarget)
    }

    const contentType = response.headers.get("content-type");
    if (contentType && contentType.indexOf("application/json") !== -1) {
      json = JSON.parse(text, reviver)
    }

    return { status, text, json }

  } catch(e: any) {
    const data = "Something went wrong and comments are unavailable"
    message(data, 'error', errorTarget)
    throw e
  }
}

function replacer(_: string, value: any) {
  return value === null || value === "" ? undefined : value
}

const DATETIME_FIELDS = ["created_at", "updated_at"]

export function reviver(key: string, value: any): any {
  return DATETIME_FIELDS.includes(key) ? new Date(value) : value
}
