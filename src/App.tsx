import {CssBaseline, ThemeProvider} from "@mui/material";
import {theme} from "./config/theme";
import { RouterProvider } from "react-router-dom";
import {router} from "./config/routing";

function App() {

  return (
      <ThemeProvider theme={theme}>
          <CssBaseline />
          <RouterProvider router={router} />
      </ThemeProvider>
  );
}

export default App;
