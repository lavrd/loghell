import React, {useEffect, useState} from 'react'

export default () => {
  const [ws, setWS] = useState(null)
  useEffect(() => {
    // TODO don't hardcode it
    const ws = new WebSocket('ws://127.0.0.1:3032/')
  }, [ws])

  return (
    <section className="container">
      <section className="grid">
        <RuleInput/>
        <RuleButton/>
        <LogsOutput/>
      </section>
    </section>
  )
}

