
import { BrowserRouter } from "react-router-dom";
import AppRouter from "@/route/router";
import "mdb-react-ui-kit/dist/css/mdb.min.css";
import "@/App.css";

function App() {
  return (
    <BrowserRouter>
      <AppRouter />
    </BrowserRouter>
  );
}

export default App;
