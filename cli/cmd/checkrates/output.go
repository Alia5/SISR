package checkrates

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

type sampleStats struct {
	mean, sd, p1, p25, p50, p75, p99 float64
	n                                int
}

func filterOutliers(data []float64) []float64 {
	if len(data) < 4 {
		return data
	}
	sorted := make([]float64, len(data))
	copy(sorted, data)
	sort.Float64s(sorted)

	n := len(sorted)
	pct := func(p float64) float64 {
		i := p * float64(n-1)
		lo := int(math.Floor(i))
		hi := int(math.Ceil(i))
		if lo == hi {
			return sorted[lo]
		}
		return sorted[lo] + (i-float64(lo))*(sorted[hi]-sorted[lo])
	}

	q1 := pct(0.25)
	q3 := pct(0.75)
	iqr := q3 - q1
	lo := q1 - 3.0*iqr
	hi := q3 + 3.0*iqr

	out := make([]float64, 0, len(data))
	for _, v := range data {
		if v >= lo && v <= hi {
			out = append(out, v)
		}
	}
	return out
}

func computeStats(data []float64) sampleStats {
	n := len(data)
	sorted := make([]float64, n)
	copy(sorted, data)
	sort.Float64s(sorted)

	var sum float64
	for _, v := range data {
		sum += v
	}
	mean := sum / float64(n)

	var vsum float64
	for _, v := range data {
		d := v - mean
		vsum += d * d
	}

	pct := func(p float64) float64 {
		i := p * float64(n-1)
		lo := int(math.Floor(i))
		hi := int(math.Ceil(i))
		if lo == hi {
			return sorted[lo]
		}
		return sorted[lo] + (i-float64(lo))*(sorted[hi]-sorted[lo])
	}

	return sampleStats{
		mean: mean,
		sd:   math.Sqrt(vsum / float64(n)),
		p1:   pct(0.01),
		p25:  pct(0.25),
		p50:  pct(0.50),
		p75:  pct(0.75),
		p99:  pct(0.99),
		n:    n,
	}
}

type tableRow struct {
	label     string
	intervals []float64
}

func maxLabelWidth(colHeader string, rows []tableRow) int {
	w := len(colHeader)
	for _, r := range rows {
		if len(r.label) > w {
			w = len(r.label)
		}
	}
	return w
}

func sepLine(lw, mw, sdw int) string {
	tw := 1 + lw + 2 + mw + 2 + sdw + (2+6)*7
	return strings.Repeat("─", tw)
}

func printTableHeader(lw int, colLabel, meanHeader, sdHeader string) {
	mw := len(meanHeader)
	sdw := len(sdHeader)
	sep := sepLine(lw, mw, sdw)
	fmt.Printf("%s\n", sep)
	fmt.Printf(
		" %-*s  %*s  %*s  %6s  %6s  %6s  %6s  %6s  %6s  %6s\n",
		lw, colLabel,
		mw, meanHeader,
		sdw, sdHeader,
		"P1", "P25", "P50", "P75", "P99", "N", "Total",
	)
	fmt.Printf("%s\n", sep)
}

func printMsRows(lw, mw, sdw int, rows []tableRow) {
	for _, r := range rows {
		filtered := filterOutliers(r.intervals)
		if len(filtered) < 2 {
			continue
		}
		st := computeStats(filtered)
		fmt.Printf(
			" %-*s  %*.3f  %*.3f  %6.3f  %6.3f  %6.3f  %6.3f  %6.3f  %6d  %6d\n",
			lw, r.label,
			mw, st.mean,
			sdw, st.sd,
			st.p1, st.p25, st.p50, st.p75, st.p99,
			st.n, len(r.intervals),
		)
	}
}

func printHzRows(lw, mw, sdw int, rows []tableRow) {
	for _, r := range rows {
		hz := make([]float64, len(r.intervals))
		for i, v := range r.intervals {
			hz[i] = 1000.0 / v
		}
		filtered := filterOutliers(hz)
		if len(filtered) < 2 {
			continue
		}
		st := computeStats(filtered)
		fmt.Printf(
			" %-*s  %*.1f  %*.1f  %6.1f  %6.1f  %6.1f  %6.1f  %6.1f  %6d  %6d\n",
			lw, r.label,
			mw, st.mean,
			sdw, st.sd,
			st.p1, st.p25, st.p50, st.p75, st.p99,
			st.n, len(r.intervals),
		)
	}
}

func printMsTable(title string, rows []tableRow) {
	const meanHeader = "Mean [ms]"
	const sdHeader = "Standard deviation [ms]"
	lw := maxLabelWidth("Controller", rows)
	mw := len(meanHeader)
	sdw := len(sdHeader)
	fmt.Printf("\n%s\n", title)
	printTableHeader(lw, "Controller", meanHeader, sdHeader)
	printMsRows(lw, mw, sdw, rows)
}

func printHzTable(title string, rows []tableRow) {
	const meanHeader = "Mean [Hz]"
	const sdHeader = "Standard deviation [Hz]"
	lw := maxLabelWidth("Controller", rows)
	mw := len(meanHeader)
	sdw := len(sdHeader)
	fmt.Printf("\n%s\n", title)
	printTableHeader(lw, "Controller", meanHeader, sdHeader)
	printHzRows(lw, mw, sdw, rows)
}

func printLatencyTable(title string, rows []tableRow) {
	const meanHeader = "Mean [ms]"
	const sdHeader = "Standard deviation [ms]"
	lw := maxLabelWidth("Real -> Virtual", rows)
	mw := len(meanHeader)
	sdw := len(sdHeader)
	fmt.Printf("\n%s\n", title)
	printTableHeader(lw, "Real -> Virtual", meanHeader, sdHeader)
	printMsRows(lw, mw, sdw, rows)
}

func printResults(results []padResult) {
	rows := make([]tableRow, 0, len(results))
	for _, r := range results {
		rows = append(rows, tableRow{label: r.name, intervals: r.intervals})
	}
	printMsTable("Polling Intervals", rows)
	printHzTable("Polling Rate", rows)
	fmt.Println()
}

func printCompareResults(pairs []comparePair) {
	var msRows, hzRows, latRows []tableRow
	for _, cp := range pairs {
		msRows = append(msRows,
			tableRow{label: cp.realName + " (Real)", intervals: cp.realIntervals},
			tableRow{label: cp.virtName + " (Virtual)", intervals: cp.virtIntervals},
		)
		hzRows = append(hzRows,
			tableRow{label: cp.realName + " (Real)", intervals: cp.realIntervals},
			tableRow{label: cp.virtName + " (Virtual)", intervals: cp.virtIntervals},
		)
		if len(cp.latencies) >= 2 {
			latRows = append(latRows, tableRow{
				label:     cp.realName + " -> " + cp.virtName,
				intervals: cp.latencies,
			})
		}
	}

	printMsTable("Polling Intervals", msRows)
	printHzTable("Polling Rate", hzRows)
	if len(latRows) > 0 {
		printLatencyTable("Steam Input Added Latency", latRows)
	}
	fmt.Println()
}
