import { Link } from "react-router-dom";
import React, { useState, useEffect } from "react";
import { BASE_URL } from "./App";
import './Nav.css';

function Nav() {
  const [visible, setVisible] = useState(true);
  const [sessions, setSessions] = useState([]);

  useEffect(() => {
    fetch(`${BASE_URL}/sessions`)
      .then(res => res.json())
      .then(res => setSessions(res))
      .catch(console.error);
  },[])
  
  return (
    <nav>
      <button onClick={() => setVisible(visible => !visible)}>Toggle</button>
      {visible && <div>
        { 
          sessions.map(session => 
            <Link key={`session-${session.sessionId}`} to={`/session/${session.sessionId}`}>
              {session.name}, {new Date(session.createTime).toLocaleString()}
            </Link>
          )
        }
      </div>}
    </nav>
  );
}

export default Nav;
