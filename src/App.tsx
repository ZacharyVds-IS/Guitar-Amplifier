import {MainScreen} from "./screens/MainScreen.tsx";
import {CssBaseline, ThemeProvider} from "@mui/material";
import {theme} from "./config/theme";

function App() {

  return (
      <ThemeProvider theme={theme}>
          <CssBaseline />
          <MainScreen/>
      </ThemeProvider>
  );
}

export default App;
