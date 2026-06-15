import { Component, type ReactNode, type ErrorInfo } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
  info: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, info: null };

  static getDerivedStateFromError(error: Error) {
    return { error, info: null };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    this.setState({ info });
    console.error("[ErrorBoundary]", error, info);
  }

  handleReload = () => {
    this.setState({ error: null, info: null });
  };

  render() {
    if (this.state.error) {
      return (
        <div className="flex items-center justify-center h-screen bg-surface-900 text-gray-100 p-8">
          <div className="glass-card rounded-xl p-8 max-w-lg w-full text-center space-y-4">
            <div className="w-12 h-12 rounded-full bg-red-950/60 border border-red-900/50 flex items-center justify-center mx-auto">
              <AlertTriangle className="w-6 h-6 text-red-accent" />
            </div>
            <h2 className="text-lg font-semibold">Something went wrong</h2>
            <p className="text-sm text-gray-400 font-mono bg-surface-800 rounded-lg p-3 text-left break-all">
              {this.state.error.message}
            </p>
            <button
              onClick={this.handleReload}
              className="inline-flex items-center gap-2 px-4 py-2 text-sm bg-accent/10 text-accent border border-accent/20 rounded-lg hover:bg-accent/20 transition-all"
            >
              <RefreshCw className="w-4 h-4" />
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
