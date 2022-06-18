import { Route, Routes, BrowserRouter } from "react-router-dom";
import Nav from "./Nav";
import Session from "./Session";

export const BASE_URL = "http://192.168.1.200:8888";

function App() {
  return (
    <BrowserRouter>
      <Nav />
      <Routes>
        <Route path="/" />
        <Route path="/session/:sessionId" element={<Session />}/>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
