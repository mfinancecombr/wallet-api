/*
 * Copyright (c) 2020, Marcelo Jorge Vieira (https://github.com/mfinancecombr)
 *
 * License: BSD 3-Clause
 */

import React from "react";

import PieChart from "./PieChart";

const Allocation = ({ data, width, height, isMoney }) => {
  if (data === undefined) return null;

  const items = Object.values(data).map((item) => {
    const total = item.costBasis + item.gain;
    return { name: item.symbol, value: total };
  });

  return (
    <React.Fragment>
      <h4>Allocation</h4>
      <div style={{ width: width, height: height }}>
        <PieChart data={items} outerRadius={100} isMoney />
      </div>
    </React.Fragment>
  );
};

export default Allocation;
