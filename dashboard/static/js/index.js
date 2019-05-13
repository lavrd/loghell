let ws = null

window.onload = () => {
  ws = new WebSocket('ws://javascript.ru/ws')
  initWebsocket()
}

window.onbeforeunload = () => {
  ws.onclose = () => {
    // disable onclose handler first
  }
  ws.close()
}

const initWebsocket = () => {
  ws.onclose = (event) => {
    if (event.wasClean) {
      alert('websocket connection closed')
    } else {
      alert('websocket connection interrupt')
    }
    alert('unknown problem with websocket')
  }

  ws.onmessage = (event) => {

  }

  ws.onerror = (error) => {
    alert('error ' + error.message)
  }
}
