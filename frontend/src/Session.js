import { useParams } from "react-router-dom";
import React, { useState, useEffect } from "react";
import { BASE_URL } from "./App";
import Plot from 'react-plotly.js';

function Session() {
  const [equity, setEquity] = useState([]);
  const [oldEquity, setOldEquity] = useState([]);
  const [orders, setOrders] = useState([]);
  const [info, setInfo] = useState(null);

  const { sessionId } = useParams();

  useEffect(() => {
    setInfo(null);
    setOrders([]);
    setEquity([]);
    setOldEquity([]);

    fetch(`${BASE_URL}/sessions/${sessionId}/more`)
      .then(res => res.json())
      .then(res => setInfo(res))
      .catch(console.error);
  }, [sessionId]);

  useEffect(() => {
    fetch(`${BASE_URL}/sessions/${sessionId}/equity/${oldEquity.length}`)
      .then(res => res.json())
      .then(res => res.map((elem) => {
        return {
          time: new Date(elem.time),
          total: elem.total,
        }
      }))
      .then(res => {
        setEquity(equity => equity === oldEquity ? [...oldEquity, ...res] : equity);
      })
      .catch(console.error);
  }, [sessionId, oldEquity]);
  
  
  useEffect(() => {
    let interval = setInterval(() => {
      setOldEquity(equity);
    }, 1000);

    return () => clearInterval(interval);
  }, [sessionId, equity]);

  const deleteData = () => {
    fetch(`${BASE_URL}/sessions/${sessionId}/delete`)
      .catch(console.error);
  }

  return (
    <main>
      <h1>Session {sessionId}</h1>
      <button onClick={deleteData}>Delete Session Data</button>

      {equity.length !== 0 && info !== null && <div>
        <h2>General Information</h2>
        <table className="info">
          <tbody>
            <tr><th>Created</th><td>{new Date(info.info.createTime).toLocaleString()}</td></tr>
            <tr><th>Strategy</th><td>{info.info.name}</td></tr>
            <tr><th>Exchange</th><td>{info.info.exchange}</td></tr>
            <tr><th>Type</th><td>{info.info.live_trading ? "Live Trading" : "Backtest"}</td></tr>
          </tbody>
        </table>
        <h2>Performance</h2>
        <table className="info">
          <tbody>
            <tr><th>Annual Rate of Return</th><td>{Math.round(info.annualRateOfReturn * 10000) / 100}%</td></tr>
            <tr><th>Annual Turnover</th><td>${Math.round(info.annualTurnover).toLocaleString()}</td></tr>
            <tr><th>Operating Margin</th><td>{Math.round(info.operatingMargin * 10000) / 100}%</td></tr>
          </tbody>
        </table>
        <Plot
          data={[
            {
              x: equity.map(entry => entry.time),
              y: equity.map(entry => entry.total),
              type: 'scatter',
              mode: 'lines',
              marker: {color: 'red'},
            },
          ]}
          layout={{
            autosize: true,
            xaxis: {
              title: "Time"
            },
            yaxis: {
              title: "Equity USD"
            }
          }}
          style={{width: "100%", height: 600}}
        />
        <h2>Risk</h2>
        <table className="info">
          <tbody>
            <tr><th>Average Daily Win</th><td>{Math.round(info.avgDailyRateOfReturn * 10000) / 100}%</td></tr>
            <tr><th>Worst Daily Loss (95% Confidence)</th><td>{Math.round(-1.65 * info.stdevDailyRateOfReturn * 10000) / 100}%</td></tr>
            <tr><th>Worst Daily Loss (99% Confidence)</th><td>{Math.round(-2.33 * info.stdevDailyRateOfReturn * 10000) / 100}%</td></tr>
          </tbody>
        </table>
        <Plot
          data={[
            {
              x: info.dailyRateOfReturns,
              type: 'histogram',
            },
          ]}
          layout={{
            autosize: true,
            xaxis: {
              title: "Daily Rate of Profit",
              tickformat: "%"
            },
            yaxis: {
              title: "Probability Distribution"
            }
          }}
          style={{width: "100%", height: 600}}
        />
      </div>}
    </main>
  );
}

export default Session;
