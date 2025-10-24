import pandas as pd
import matplotlib.pyplot as plt
import numpy as np


def plot_log_data(csv_file="log.csv", output_file="graph.svg", latest_n=30):
    try:
        df = pd.read_csv(csv_file)
    except FileNotFoundError:
        print(f"error: File '{csv_file}' not found.")
        return

    # Get the latest 30 entries
    recent_df = df.tail(latest_n).copy()

    # Filter only instruction-related columns (columns ending with '-inst')
    inst_columns = [col for col in recent_df.columns if col.endswith('-inst')]

    if not inst_columns:
        print("error: No instruction columns found in the data.")
        return

    # Calculate ratios relative to the oldest commit (first row) for each column
    for col in inst_columns:
        baseline_value = recent_df[col].iloc[0]  # First (oldest) commit value
        if baseline_value != 0:
            recent_df[col] = recent_df[col] / baseline_value
        else:
            print(
                f"warning: Baseline value for {col} is zero. Skipping normalization.")

    # Create the graph with logarithmic y-axis
    fig, ax = plt.subplots(figsize=(15, 8))

    # Plot instruction columns
    color_map = plt.get_cmap('tab10')
    for i, col in enumerate(inst_columns):
        ax.plot(recent_df['commit'], recent_df[col],
                label=col, color=color_map(i), marker='o')

    # Set logarithmic scale for y-axis
    ax.set_yscale('log')

    # Fix y-axis range: 0.5 (50%) to 2.0 (200%)
    ax.set_ylim(0.5, 2.0)
    
    # Set custom y-axis ticks from 0.5 to 2.0 in 0.1 increments
    y_ticks = np.arange(0.5, 2.1, 0.1)
    ax.set_yticks(y_ticks)
    ax.set_yticklabels([f"{tick:.1f}" for tick in y_ticks])

    ax.set_xlabel('Commit')
    ax.set_ylabel('Performance Ratio (relative to oldest commit)')

    # Set x-axis labels to first 7 characters of commit hash
    ax.set_xticks(range(len(recent_df['commit'])))
    ax.set_xticklabels([c[:7] for c in recent_df['commit']],
                       rotation=90, ha='right')

    # Graph title and legend
    plt.title(
        f'Instruction Performance Ratios for Last {latest_n} Commits (Log Scale)')
    ax.legend(loc='upper left')

    # Add grid for better readability on log scale
    ax.grid(True, alpha=0.3)

    # Adjust layout
    fig.tight_layout()

    # Save as SVG file
    plt.savefig(output_file)
    print(f"Graph successfully saved as '{output_file}'.")


if __name__ == "__main__":
    plot_log_data()
