let ws = null

window.onload = () => {
  console.log(wsPort)
  ws = new WebSocket('ws://127.0.0.1:' + wsPort + '/?rule=!level=debug@debug')
  initWebsocket()
}

window.onbeforeunload = () => {
  ws.onclose = () => {
    // disable onclose handler first
  }
  ws.close(1001, 'browser page closed')
}

const initWebsocket = () => {
  ws.onclose = (event) => {
    if (event.wasClean) {
      console.log('websocket connection closed')
    } else {
      console.log('websocket connection interrupt')
    }
    console.log('unknown problem with websocket')
  }

  ws.onmessage = (event) => {

  }

  ws.onerror = (error) => {
    console.log('error ' + error.message)
  }
}
