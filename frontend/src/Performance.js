import * as React from "react";
import { useState, useEffect } from "react";
import { fetchStart, fetchEnd, Loading, Error } from "react-admin";
import { useDispatch } from "react-redux";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

export const Performance = (props) => {
  const dispatch = useDispatch();
  const [data, setData] = useState();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState();

  useEffect(() => {
    setLoading(true);
    dispatch(fetchStart());
    let query = "";
    if (props.portfolio !== undefined) {
      query = "?oid=" + props.portfolio;
    }
    fetch("http://localhost:8000/api/v1/portfolios/performance" + query, {
      method: "GET",
      cache: "no-cache",
    })
      .then((response) => response.json())
      .then((data) => {
        console.log(data);
        setData(data);
        setLoading(false);
      })
      .catch((error) => {
        setError(error);
        setLoading(false);
      })
      .finally(() => {
        setLoading(false);
        dispatch(fetchEnd());
      });
  }, []);

  if (loading) return <Loading />;
  if (error) return <Error />;
  if (!data) return null;

  return (
    <ResponsiveContainer width="90%" aspect={4.0 / 2.0}>
      <AreaChart
        data={data}
        margin={{
          top: 50,
          right: 30,
          left: 20,
          bottom: 70,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="name" angle={-45} textAnchor="end" />
        <YAxis tickFormatter={(tick) => tick + "%"} />
        <Tooltip />
        <Area
          connectNulls
          type="monotone"
          dataKey="percentual_gain"
          stroke="#82ca9d"
          fill="#82ca9d"
        />
      </AreaChart>
    </ResponsiveContainer>
  );
};
