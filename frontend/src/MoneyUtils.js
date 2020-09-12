/*
 * Copyright (c) 2020, Marcelo Jorge Vieira (https://github.com/mfinancecombr)
 *
 * License: BSD 3-Clause
 */
import * as React from "react";
import { makeStyles } from "@material-ui/styles";

export const convertToBRLMoney = (num) => {
  if (num === undefined || num === null) {
    return "N/A";
  }

  num = parseFloat(num.toFixed(2));
  return num.toLocaleString("pt-br", {
    style: "currency",
    currency: "BRL",
  });
};

export const convertToBRLFloat = (num) => {
  if (num === undefined || num === null) {
    return "N/A";
  }

  num = parseFloat(num.toFixed(2));
  return num.toLocaleString("pt-br", { minimumFractionDigits: 2 });
};

export const MoneyField = ({ source, record, calculate }) => {
  let number;
  if (calculate !== undefined) {
    number = calculate(record);
  } else {
    number = record[source];
  }

  return <span>{convertToBRLMoney(number)}</span>;
};

const useStyles = makeStyles((theme) => ({
  gainColoring: (gain) => ({
    color: parseFloat(gain, 10) >= 0 ? "green" : "red",
    flex: 1,
  }),
}));

const ColoredGain = ({ gain, children }) => {
  const classes = useStyles(gain);
  return <span className={classes.gainColoring}>{children}</span>;
};

export const GainField = ({ source, record }) => {
  let gain = (record.gain / record.cost_basis) * 100;
  return <ColoredGain gain={gain}>{convertToBRLFloat(gain)}%</ColoredGain>;
};
