import axios from 'axios';
import { useState } from 'react';
import './App.css';

function App() {
  const [count, setCount] = useState(0);
  const [res, setRes] = useState<String>("");

  axios.get("/api/foo")
    .then(response => {
      console.log(response);
      setRes(response.data);
    })
    .catch(error => {
      console.log(error);
    })
    .finally(() => {});

  return (
    <div className="App">
      MESSAGE: {res}
    </div>
  )
}

export default App
