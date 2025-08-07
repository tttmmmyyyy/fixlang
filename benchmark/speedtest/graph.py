import pandas as pd
import matplotlib.pyplot as plt


def plot_log_data(csv_file="log.csv", output_file="graph.svg", latest_n=30):
    try:
        df = pd.read_csv(csv_file)
    except FileNotFoundError:
        print(f"error: File '{csv_file}' not found.")
        return

    # Get the latest 30 entries
    recent_df = df.tail(latest_n).copy()

    # Get all columns except 'commit' (data columns)
    data_columns = [col for col in recent_df.columns if col != 'commit']

    # Create the graph
    fig, ax1 = plt.subplots(figsize=(15, 8))

    # List to store information for legend
    lines = []
    labels = []

    # Use ax1 as Y-axis for the first data column
    color_map = plt.get_cmap('tab20')  # Automatically assign line colors

    # Plot the first data column
    col = data_columns[0]
    line1, = ax1.plot(recent_df['commit'], recent_df[col],
                      label=col, color=color_map(0//2), marker='o')
    lines.append(line1)
    labels.append(col)

    ax1.set_xlabel('Commit')
    ax1.set_ylabel('')  # Make y-axis label empty
    ax1.tick_params(axis='y', left=False, right=False,
                    labelleft=False, labelright=False, labelcolor=color_map(0//2))

    # Create new Y-axes using twinx() for remaining data columns
    for i, col in enumerate(data_columns[1:], start=1):
        ax = ax1.twinx()

        # Position Y-axes on the right side to prevent graph overlap
        ax.spines['right'].set_position(('outward', 60 * (i - 1)))

        line_i, = ax.plot(recent_df['commit'], recent_df[col],
                          label=col, color=color_map(i//2), marker='o')
        lines.append(line_i)
        labels.append(col)

        # Hide y-axis ticks and labels
        ax.set_ylabel('')  # Make y-axis label empty
        ax.tick_params(axis='y', left=False, right=False,
                       labelleft=False, labelright=False)

        # Move Y-axis labels to prevent graph overlap
        ax.spines['right'].set_color(color_map(i//2))

    # Set x-axis labels to first 7 characters of commit hash
    ax1.set_xticks(range(len(recent_df['commit'])))
    ax1.set_xticklabels([c[:7]
                         for c in recent_df['commit']], rotation=90, ha='right')

    # Graph title and legend
    plt.title(f'Performance Metrics for Last {latest_n} Commits')
    # Create legend using lines and labels obtained from all axes
    fig.legend(lines, labels, loc='upper left', bbox_to_anchor=(0.05, 0.95))
    # Adjust layout
    fig.tight_layout()

    # Save as SVG file
    plt.savefig(output_file)
    print(f"Graph successfully saved as '{output_file}'.")


if __name__ == "__main__":
    plot_log_data()
