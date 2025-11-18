/**
 * Calculate 3 standard deviation width for box spread strikes
 * Formula: width = 3 × underlying_price × implied_vol × sqrt(days_to_expiry / 365)
 *
 * @param underlyingPrice Current price of underlying asset
 * @param daysToExpiry Days until expiration
 * @param impliedVol Implied volatility (default: 0.20 = 20%)
 * @returns Width in price points for 3 standard deviations
 */
export function calculate3StdWidth(
  underlyingPrice: number,
  daysToExpiry: number,
  impliedVol: number = 0.20
): number {
  const timeToExpiry = daysToExpiry / 365;
  const stdDev = underlyingPrice * impliedVol * Math.sqrt(timeToExpiry);
  return 3 * stdDev;
}

/**
 * Round strike to nearest valid increment
 * Common increments: $0.50, $1, $2.50, $5, $10
 *
 * @param strike Strike price to round
 * @param increment Strike increment (default: $5 for most indices)
 * @returns Rounded strike price
 */
export function roundToStrikeIncrement(strike: number, increment: number = 5): number {
  return Math.round(strike / increment) * increment;
}

/**
 * Calculate box spread strikes with selected strike as center
 *
 * @param centerStrike Selected strike (middle of box spread)
 * @param width Total width of box spread
 * @param increment Strike increment for rounding
 * @returns Object with lower and upper strikes
 */
export function calculateBoxSpreadStrikes(
  centerStrike: number,
  width: number,
  increment: number = 5
): { lowerStrike: number; upperStrike: number } {
  const halfWidth = width / 2;
  const lowerStrike = roundToStrikeIncrement(centerStrike - halfWidth, increment);
  const upperStrike = roundToStrikeIncrement(centerStrike + halfWidth, increment);

  // Ensure lower < upper
  if (lowerStrike >= upperStrike) {
    return {
      lowerStrike: roundToStrikeIncrement(centerStrike - increment, increment),
      upperStrike: roundToStrikeIncrement(centerStrike + increment, increment)
    };
  }

  return { lowerStrike, upperStrike };
}
