const HOST = document.getElementById('besedka')?.dataset.api

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

export function createButton(text: string, className?: string, attributes?: {}): HTMLButtonElement {
  const button = createElement<HTMLButtonElement>('button', className, attributes)
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

export function debounce(fn: Function, delay = 500) {
  let timeout: number

  return () => {
    if (timeout) clearTimeout(timeout)

    timeout = setTimeout(() => fn(), delay)
  }
}

export function timeago(date: Date): string {
  const now = new Date().getTime()
  const secondsAgo = (now - date.getTime()) / 1000
  const minutesAgo = Math.floor(secondsAgo / 60)

  if (minutesAgo < 2) return "just now"
  if (minutesAgo < 60) return `${minutesAgo} minutes ago`

  const hoursAgo = Math.floor(minutesAgo / 60)
  if (hoursAgo == 1) return "an hour ago"
  if (hoursAgo < 12) return `${hoursAgo} hours ago`
  if (hoursAgo < 24) return "yesterday"

  const daysAgo = Math.floor(hoursAgo / 24)
  if (daysAgo == 1) return "yesterday"
  if (daysAgo < 7) return `${daysAgo} days ago`
  if (daysAgo < 14) return "a week ago"

  const weeksAgo = Math.floor(daysAgo / 7)
  if (daysAgo < 30) return `${weeksAgo} weeks ago`

  const monthsAgo = Math.floor(daysAgo / 30)
  if (monthsAgo == 1) return "a month ago"
  if (monthsAgo < 12) return `${monthsAgo} months ago`
  if (monthsAgo < 15) return "a year ago"
  if (monthsAgo < 23) return "more than a year ago"

  const yearsAgo = Math.floor(monthsAgo / 12)
  return `${yearsAgo} years ago`
}
