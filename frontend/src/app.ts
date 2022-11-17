import {ConfigRequest} from "./types"

export default class App {
  container: HTMLElement
  configRequest?: ConfigRequest

  constructor(container: HTMLElement, configRequest?: ConfigRequest) {
    this.container = container
    this.configRequest = configRequest
  }
}
