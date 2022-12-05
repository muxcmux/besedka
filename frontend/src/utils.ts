const HOST = "http://besedka.com:6353"

export function safeParse(text: string | undefined, def = {}) {
  try {
    return JSON.parse(text || '{}')
  } catch {
    return def
  }
}

export async function post(endpoint: string, data: {}) {
  const body = JSON.stringify(data, replacer)
  return await fetch(`${HOST}${endpoint}`, {
    body,
    method: "POST",
    mode: "cors",
    headers: { "Content-Type": "application/json" }
  })
}

function replacer(_: string, value: any) {
  return value === null ? undefined : value
}

const DATETIME_FIELDS = ["created_at", "updated_at"]

export function reviver(key: string, value: any): any {
  return DATETIME_FIELDS.includes(key) ? new Date(value) : value
}
