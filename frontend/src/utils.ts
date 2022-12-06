const HOST = "http://besedka.com:6353"

export function safeParse(text: string | undefined, def = {}) {
  try {
    return JSON.parse(text || '{}')
  } catch {
    return def
  }
}

export function message(msg: string, klass = 'error') {
  document.getElementById('besedka-message')!.innerHTML = `
    <div class="besedka-${klass}">${msg}</li>
  `
}

export function setToken(token: string) {
  window.localStorage.setItem('__besedka_token', token)
}

export function getToken(): string | null {
  return window.localStorage.getItem('__besedka_token')
}

export async function post<T>(endpoint: string, data: {}): Promise<{ status: number, text: string, json: T | null } > {
  const body = JSON.stringify(data, replacer)
  try {
    const response = await fetch(`${HOST}${endpoint}`, {
      body,
      method: "POST",
      mode: "cors",
      headers: { "Content-Type": "application/json" }
    })

    const status = response.status
    let text = await response.text()
    let json: T | null = null

    if (response.status > 399 && response.status != 404) {
      message(text)
    }

    const contentType = response.headers.get("content-type");
    if (contentType && contentType.indexOf("application/json") !== -1) {
      json = JSON.parse(text, reviver)
    }

    return { status, text, json }

  } catch(e: any) {
    const data = "Something went wrong and comments are unavailable"
    message(data)
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
