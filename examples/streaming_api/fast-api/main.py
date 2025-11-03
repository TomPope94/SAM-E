from fastmcp import FastMCP

# Create MCP server
mcp = FastMCP("Hello World MCP Server", stateless_http=True)

# Use the MCP app directly - it's already a FastAPI app
app = mcp.http_app()


# Define a simple tool
@mcp.tool()
def greet(name: str) -> str:
    """
    A simple greeting tool that says hello to someone.

    Args:
        name: The name of the person to greet

    Returns:
        A friendly greeting message
    """
    print(f"Greeting tool called with name: {name}")
    return f"Hello, {name}! Welcome to the MCP server."


@mcp.tool()
def add_numbers(a: int, b: int) -> int:
    """
    Add two numbers together.

    Args:
        a: First number
        b: Second number

    Returns:
        The sum of a and b
    """
    print(f"Adding {a} + {b}")
    return a + b


# Define a simple resource
@mcp.resource("greeting://hello")
def hello_resource() -> str:
    """A simple hello world resource"""
    print("Hello resource accessed")
    return "Hello from the MCP resource!"


# Print available routes on startup
# @app.on_event("startup")
# async def startup_event():
#     print("=" * 50)
#     print("MCP Server Started - Available Routes:")
#     print("=" * 50)
#     for route in app.routes:
#         if hasattr(route, "methods") and hasattr(route, "path"):
#             print(f"  {', '.join(route.methods):10} {route.path}")
#     print("=" * 50)
